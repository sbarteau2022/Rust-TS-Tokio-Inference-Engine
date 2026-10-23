"""Offline independent HTTP-control accounting; Rust owns model execution and wire validation."""
from pathlib import Path
import json, math, re, subprocess, sys, statistics
ROOT=Path(__file__).resolve().parents[1]

def require(ok,message):
    if not ok:raise AssertionError(message)
def close(a,b):
    require(isinstance(a,(int,float)) and isinstance(b,(int,float)) and math.isfinite(a) and math.isfinite(b) and math.isclose(a,b,rel_tol=1e-10,abs_tol=1e-6),f'arithmetic mismatch {a} != {b}')
def fields(text):
    out={}
    for item in text.split(', '):
        key,value=item.split(': ')
        require(key not in out,'duplicate counter')
        require(value in ['true','false'] or value.isdecimal(),'invalid counter')
        out[key]=value=='true' if value in ['true','false'] else int(value)
    return out

def drain(log,successful,rejected,profile):
    lines=[s for s in log.splitlines() if s.startswith('DRAIN ')]
    require(len(lines)==1,'exactly one drain required')
    match=re.fullmatch(r'DRAIN http=Stats \{ ([^{}]+) \} scheduler=Stats \{ ([^{}]+) \} native_bytes=(\d+) audit_epochs=(\d+)',lines[0])
    require(match is not None,'invalid drain')
    http,scheduler=fields(match[1]),fields(match[2])
    require(all(type(v) is int for v in http.values()),'HTTP counter types')
    require(all(type(v) is (bool if k=='admission_closed' else int) for k,v in scheduler.items()),'scheduler counter types')
    require(int(match[3])==0,'native ownership remains')
    for key in ['active_connections','active_producers','deadlines','disconnected','stopped','producer_errors']:require(http[key]==0,f'HTTP {key}')
    for key in ['failed','cancelled','memory_used']:require(scheduler[key]==0,f'scheduler {key}')
    require(scheduler['admission_closed'] is False,'admission unexpectedly closed')
    require(scheduler['admitted']==scheduler['completed']==http['queued_finishes']==successful,'receipt accounting')
    require(http['rejected_requests']==rejected and http['accepted_connections']==successful+rejected,'connection/rejection accounting')
    require((int(match[4])>0)==(profile=='mtp-audited'),'audit clock/profile mismatch')
    return {'http':http,'scheduler':scheduler,'native_bytes':int(match[3]),'audit_epochs':int(match[4])}

def row_times(row,start,end):
    arrivals,publication=row['arrivals'],row['publication']
    require(len(arrivals)==len(publication)==len(row['tokens'])>0,'timing count')
    require(start<=row['start']<=row['metadataArrival']<=arrivals[0]<=arrivals[-1]<=row['end']<=end,'request clock envelope')
    require(all(math.isfinite(t) for t in [row['start'],row['end'],*arrivals,*publication]),'nonfinite clock')
    require(all(a<=b for a,b in zip(arrivals,arrivals[1:])) and all(a<=b for a,b in zip(publication,publication[1:])),'clock regression')
    require(publication[0]>=0,'negative publication')

def arithmetic(cell,rows):
    start,end=cell['start'],cell['end'];require(end>start,'cell envelope')
    for row in rows:row_times(row,start,end)
    tokens=sum(len(row['tokens']) for row in rows);require(cell['tokens']==tokens,'cell token accounting')
    close(cell['wall_seconds'],(end-start)/1000);close(cell['aggregate_tokens_per_second'],tokens*1000/(end-start))
    ttft=[row['arrivals'][0]-row['start'] for row in rows]
    gaps=[b-a for row in rows for a,b in zip(row['arrivals'],row['arrivals'][1:])]
    for label,values in [('ttft_ms',ttft),('client_callback_itl_ms',gaps)]:
        require(cell[label]['count']==len(values),'distribution count');close(cell[label]['mean'],statistics.mean(values));close(cell[label]['p95'],sorted(values)[math.ceil(len(values)*.95)-1])
    duration=sum(row['arrivals'][-1]-row['arrivals'][0] for row in rows)
    close(cell['post_first_tokens_per_lane_second'],sum(len(row['tokens'])-1 for row in rows)*1000/duration)
    ordered=sorted(t for row in rows for t in row['arrivals'])
    require([w['tokens'] for w in cell['windows']]==[n for n in [1000,1500,2000] if n<=tokens],'macro window coverage')
    for window in cell['windows']:
        n=window['tokens'];close(window['elapsed_seconds'],(ordered[n-1]-start)/1000);close(window['tokens_per_second'],n/window['elapsed_seconds'])

def archive(path,row,reference,log):
    # Independent numeric extraction only after the production Rust verifier has accepted the file.
    import struct
    result=subprocess.run([str(ROOT/'target/release/rust-server'),'stream-check',str(path)],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=15)
    require(result.returncode==0,f'invalid archive {path}')
    raw=path.read_bytes();frames=[];at=0
    while at<len(raw):
        size=struct.unpack_from('<I',raw,at)[0];b=raw[at:at+4+size];at+=4+size
        table=4+struct.unpack_from('<I',b,4)[0];vt=table-struct.unpack_from('<i',b,table)[0];length=struct.unpack_from('<H',b,vt)[0]
        def location(i):
            return struct.unpack_from('<H',b,vt+4+2*i)[0] if 4+2*i<length else 0
        def scalar(i,fmt):
            off=location(i);return struct.unpack_from(fmt,b,table+off)[0] if off else 0
        def text(i):
            off=location(i)
            if not off:return ''
            p=table+off;p+=struct.unpack_from('<I',b,p)[0];n=struct.unpack_from('<I',b,p)[0];return b[p+4:p+4+n].decode('utf-8')
        frames.append({'kind':scalar(1,'<B'),'sequence':scalar(2,'<Q'),'token':scalar(4,'<I'),'publication':scalar(5,'<Q')/1000,'identity':text(7),'profile':text(8)})
    require(frames[0]['identity']==reference['identity'] and frames[0]['profile']==row['metadata']['profile'],'metadata identity/profile')
    require(frames[-1]['kind']==2,'generation archive missing Finish')
    tokens=[f for f in frames if f['kind']==1]
    require([f['token'] for f in tokens]==row['tokens']==reference['tokens'][row['prompt']][:len(tokens)],'archive/reference mismatch')
    for frame,t in zip(tokens,row['publication']):close(frame['publication'],t)
    return frames[-1]['sequence']

def verify(directory):
    directory=directory.resolve();summary=json.loads((directory/'summary.json').read_text());options=summary['options'];reference=json.loads((ROOT/'evidence/native-mtp-stream-matrix/raw/references.json').read_text())
    require(summary['pass'] and options['profiles']==['mtp-audited','mtp','target'] and options['arrival']=='closed+open' and options['tokens']==2000,'wrong control contract')
    require(len(summary['cells'])==6 and len(summary['open_loop'])==3 and summary['total_closed_tokens']==12000,'matrix coverage')
    report={'pass':True,'closed_tokens':0,'open_success_tokens':0,'profiles':[]}
    with (directory.parent/'independent-verification.log').open('w') as log:
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
            if profile!='target':
                ledger=json.loads((base/'acceptance.json').read_text());require(0<ledger['accepted']<=ledger['proposed'] and ledger['failures']==ledger['unfinished']==0,'proposer accounting');close(ledger['fraction'],ledger['accepted']/ledger['proposed'])
                require(ledger['committed']<=sum(len(row['tokens']) for row in allrows),'proposer overcount')
            report['open_success_tokens']+=load['tokens'];report['profiles'].append({'profile':profile,'successful_requests':len(allrows),'open_accepted':len(successful),'open_rejected':rejected,'drain':state})
    (directory.parent/'independent-verification.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS independent clocks, archives, target parity, rejection/receipt accounting and native drain')

def self_test():
    good='DRAIN http=Stats { active_connections: 0, active_producers: 0, accepted_connections: 3, rejected_requests: 1, queued_finishes: 2, deadlines: 0, disconnected: 0, stopped: 0, producer_errors: 0, connection_errors: 0 } scheduler=Stats { admitted: 2, completed: 2, failed: 0, cancelled: 0, iterations: 3, memory_used: 0, memory_peak: 100, admission_closed: false } native_bytes=0 audit_epochs=3'
    drain(good,2,1,'mtp-audited')
    for bad in [good.replace('native_bytes=0','native_bytes=1'),good.replace('completed: 2','completed: 1'),good.replace('admission_closed: false','admission_closed: true'),good.replace('failed: 0','failed: 1'),good.replace('producer_errors: 0','producer_errors: 1'),good+'\n'+good]:
        try:drain(bad,2,1,'mtp-audited')
        except AssertionError:pass
        else:raise AssertionError('corrupt drain accepted')
    try:row_times({'start':0,'metadataArrival':1,'end':4,'arrivals':[2,1],'publication':[1,2],'tokens':[1,2]},0,5)
    except AssertionError:pass
    else:raise AssertionError('regressed clock accepted')
    print('PASS independent verifier corruption controls')
if __name__=='__main__':
    if sys.argv[1:] == ['--self-test']:self_test()
    else:verify(Path(sys.argv[1]))
