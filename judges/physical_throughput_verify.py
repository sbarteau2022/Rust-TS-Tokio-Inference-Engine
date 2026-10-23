"""Independent offline physical target benchmark qualification; imports shared arithmetic/wire checks."""
from pathlib import Path
import json,re,sys,math,statistics
from http_controls_verify import ROOT,require,close,arithmetic,archive,drain

def verify_packing(profile,packing,log):
    require(profile in ['target','target-physical'],'unknown target profile')
    observed=re.findall(r'BATCH_METRICS BatchCounters \{ packed_calls: (\d+), packed_rows: (\d+), scalar_calls: (\d+) \}',log)
    require(len(observed)==1,'one packing receipt')
    require([packing[k] for k in ['packed_calls','packed_rows','scalar_calls']]==list(map(int,observed[0])),'packing receipt mismatch')
    calls,packed,scalar=map(int,observed[0])
    require(scalar>0,'missing scalar prefill/control work')
    require(calls==packed==0 if profile=='target' else calls>0 and 2*calls<=packed<=3*calls,'invalid physical coverage')

def self_test():
    def run(profile,calls,rows,scalar,artifact=None,duplicate=False):
        packing={'packed_calls':calls,'packed_rows':rows,'scalar_calls':scalar} if artifact is None else artifact
        log=f'BATCH_METRICS BatchCounters {{ packed_calls: {calls}, packed_rows: {rows}, scalar_calls: {scalar} }}'
        verify_packing(profile,packing,log+'\n'+log if duplicate else log)
    run('target',0,0,10)
    run('target-physical',2,4,10)
    run('target-physical',2,6,10)
    for profile,calls,rows,scalar in [('target',1,2,10),('target-physical',0,0,10),('target-physical',2,3,10),('target-physical',2,7,10),('target-physical',2,6,0)]:
        try:run(profile,calls,rows,scalar)
        except AssertionError:pass
        else:raise AssertionError('corrupt packing accepted')
    for kwargs in [{'artifact':{'packed_calls':3,'packed_rows':6,'scalar_calls':10}},{'duplicate':True}]:
        try:run('target-physical',2,6,10,**kwargs)
        except AssertionError:pass
        else:raise AssertionError('mismatched/duplicate receipt accepted')
    print('PASS physical coverage corruption controls')

def control_contract(summary,allow_single=False):
    options=summary['options']
    allowed=[['target'],['target-physical']] if allow_single else [['target','target-physical'],['target-physical','target']]
    require(summary['pass'] and options['profiles'] in allowed and options['arrival']=='closed+open' and options['tokens']==2000,'wrong control contract')
    if allow_single:
        require(options.get('layerGroup') in (1,4,10,40) and options.get('recurrentKernel') in ('reference','fused'),'explicit native candidate flags required')
    n=len(options['profiles'])
    require(len(summary['cells'])==2*n and len(summary['open_loop'])==n and summary['total_closed_tokens']==4000*n,'matrix coverage')

def verify(directory,allow_single=False):
    directory=directory.resolve();summary=json.loads((directory/'summary.json').read_text());options=summary['options'];reference=json.loads((ROOT/'evidence/native-mtp-stream-matrix/raw/references.json').read_text())
    control_contract(summary,allow_single)
    report={'pass':True,'scope':'single candidate absolute measurement; no comparative speedup claim' if allow_single else 'two-profile controlled matrix','closed_tokens':0,'open_success_tokens':0,'profiles':[]}
    with (directory/'independent-verification.log').open('w') as log:
        for profile in options['profiles']:
            base=directory/profile;cells=[c for c in summary['cells'] if c['profile']==profile];loads=[c for c in summary['open_loop'] if c['profile']==profile]
            require([c['lanes'] for c in cells]==[1,3] and len(loads)==1,'lane/profile coverage')
            rows=[]
            for cell in cells:
                require(cell['tokens']==2000 and len(cell['requests'])==32,'fixed window budget')
                for i,row in enumerate(cell['requests']):require(row['ordinal']==cell['lanes']*10000+i and row['prompt']==i%4 and len(row['tokens'])==(64 if i<31 else 16),'closed workload identity/budget')
                arithmetic(cell,cell['requests']);rows+=cell['requests'];report['closed_tokens']+=cell['tokens']
            load=loads[0];attempts=load['attempts'];require(len(attempts)==load['offered_requests']==16 and load['offered_tokens']==512,'open attempt accounting')
            successful=[];rejected=0
            for i,a in enumerate(attempts):
                close(a['scheduled'],load['start']+i*250);require(a['started']>=a['scheduled'] and a['end']>=a['started'],'arrival/response clocks')
                if a['status']=='success':
                    require(a['row']['ordinal']==90000+i and a['row']['prompt']==i%4 and len(a['row']['tokens'])==32,'open identity/output count');successful.append(a['row'])
                else:
                    require(a['status']=='overloaded' and a['error']=='HttpStatusError: HTTP 503','unexpected failure')
                    require((base/f'request-{90000+i}.amst').read_bytes()==b'','rejection unexpectedly delivered frames');rejected+=1
            require(len(successful)==load['successful_requests']>0 and rejected==load['rejected_503']>0 and load['failed_requests']==0,'open outcomes')
            require(1<load['client_outstanding_high_water']<=16,'bounded independent arrivals')
            active=peak=0
            for _,delta in sorted([(a['started'],1) for a in attempts]+[(a['end'],-1) for a in attempts]):active+=delta;peak=max(peak,active)
            require(active==0 and peak==load['client_outstanding_high_water'],'outstanding client accounting')
            arithmetic(load,successful)
            for key,values in [('dispatch_jitter_ms',[a['started']-a['scheduled'] for a in attempts]),('completion_latency_ms',[a['end']-a['started'] for a in attempts])]:
                close(load[key]['mean'],statistics.mean(values));close(load[key]['p95'],sorted(values)[math.ceil(len(values)*.95)-1])
            warm=json.loads((base/'warmup.json').read_text());require(len(warm['tokens'])==8 and warm['ordinal']==-1 and warm['prompt']==0,'warmup count')
            allrows=[warm,*rows,*successful];sequences=set()
            for row in allrows:
                sequence=archive(base/f'request-{row["ordinal"]}.amst',row,reference,log);require(sequence not in sequences,'cross-request sequence reuse');sequences.add(sequence)
            state=drain((base/'server.log').read_text(),len(allrows),rejected,profile)
            packing=json.loads((base/'packing.json').read_text())
            verify_packing(profile,packing,(base/'server.log').read_text())
            for row in allrows:
                require(('physical_tensor_batching='+str(profile=='target-physical').lower()) in row['metadata']['profile'],'wrong native physical mode')
                require('execution=Target' in row['metadata']['profile'],'speculation unexpectedly active')
                require(f"layer_group={options.get('layerGroup',1)} " in row['metadata']['profile'] if 'layerGroup' in options else True,'wrong completion group')
                require(f"recurrent_kernel={options.get('recurrentKernel','reference')} " in row['metadata']['profile'] if 'recurrentKernel' in options else True,'wrong recurrent kernel')
            report['open_success_tokens']+=load['tokens'];report['profiles'].append({'profile':profile,'successful_requests':len(allrows),'open_accepted':len(successful),'open_rejected':rejected,'drain':state})
    (directory/'independent-verification.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS independent clocks, archives, target parity, rejection/receipt accounting and native drain')

if __name__=='__main__':
    require(len(sys.argv) in (2,3),'physical_throughput_verify DIRECTORY [--single]|--self-test')
    if sys.argv[1]=='--self-test':self_test()
    else:
        require(len(sys.argv)==2 or sys.argv[2]=='--single','unknown verifier mode')
        verify(Path(sys.argv[1]),len(sys.argv)==3)
