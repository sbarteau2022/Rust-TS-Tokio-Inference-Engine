/** Offline native HTTP qualification. No model execution takes place in JS. */
import {spawn,execFileSync} from 'node:child_process';import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';import {createHash} from 'node:crypto';import {createServer} from 'node:net';import {fileURLToPath} from 'node:url';import {setTimeout as delay} from 'node:timers/promises';
import {generate,HttpStatusError} from './http-client.js';import {summarize,artifactJson,distribution,type TimedRequest} from './http-metrics.js';import {encode,type Frame} from './stream.js';import {benchOptions,profileLabels,executionFlags,wavePlan,openLoop,waitForExit} from './http-bench-config.js';
const out=process.argv[2],options=benchOptions(process.argv.slice(3));if(!out||existsSync(out))throw Error('http-bench NEW_DIRECTORY [explicit flags]');
const refs=JSON.parse(readFileSync('evidence/native-mtp-stream-matrix/raw/references.json','utf8')) as {identity:string;tokens:number[][]};const prompts=readFileSync('fixtures/native-speculative-prompts.tsv','utf8').trim().split('\n').map(s=>s.slice(s.indexOf('\t')+1));if(prompts.length!==refs.tokens.length)throw Error('reference count');
const now=()=>Number(process.hrtime.bigint())/1e6,binary=fileURLToPath(new URL('../target/release/rust-server',import.meta.url));mkdirSync(out,{recursive:true});
writeFileSync(out+'/benchmark.json',artifactJson({argv:process.argv,options,benchmark_source_sha256:createHash('sha256').update(readFileSync(fileURLToPath(import.meta.url))).digest('hex'),native_binary_sha256:createHash('sha256').update(readFileSync(binary)).digest('hex')}));
writeFileSync(out+'/host.txt',execFileSync('sh',['-c','sw_vers; sysctl hw.memsize hw.ncpu; ps -axo pid,pcpu,rss,comm'],{encoding:'utf8'}).split('\n').map(s=>s.trimEnd()).join('\n'));
const results:unknown[]=[],loads:unknown[]=[];
for(const profile of options.profiles){
 const dir=out+'/'+profile;mkdirSync(dir);const listener=createServer();await new Promise<void>((ok,no)=>{listener.once('error',no);listener.listen(0,'127.0.0.1',ok)});const port=(listener.address() as {port:number}).port;await new Promise<void>(ok=>listener.close(()=>ok()));
 const args=['native-mtp-serve',`127.0.0.1:${port}`,'1000','8',...executionFlags(profile),'--layer-group',String(options.layerGroup),'--recurrent-kernel',options.recurrentKernel,'--execution-timing',options.executionTiming,'--graph-mode',options.graphMode,'--expert-kernel',options.expertKernel,'--sampling',options.sampling,'--draft-width',String(options.draftWidth),'--stream-capacity','1','--qualification-trials','64','--probe-epochs','2','--initial-mass','2147483647','--admission-mass','268435455','--reward-mass','536870911'];
 writeFileSync(dir+'/command.json',artifactJson({binary,args,binary_sha256:createHash('sha256').update(readFileSync(binary)).digest('hex'),reference_identity:refs.identity}));
 const child=spawn(binary,args,{stdio:['ignore','pipe','pipe']});let log='',closed=false,code:number|null=null;const exited=new Promise<void>(ok=>child.on('close',c=>{closed=true;code=c;ok()}));for(const s of [child.stdout,child.stderr])s.on('data',b=>{log+=b.toString();writeFileSync(dir+'/server.log',log)});child.on('error',e=>{log+='\nSPAWN_ERROR '+String(e);writeFileSync(dir+'/server.log',log)});
 try{
  const until=now()+180000;while(!log.includes('READY http://')){if(closed)throw Error('server exited before ready');if(now()>until)throw Error('ready timeout; owned process still requires drain');await delay(100);}
  const origin=`http://127.0.0.1:${port}`;
  async function request(ordinal:number,prompt:number,n:number):Promise<TimedRequest&{ordinal:number;prompt:number;tokens:number[];metadata:Frame;metadataArrival:number}>{
   const arrivals:number[]=[],publication:number[]=[],tokens:number[]=[],frames:Frame[]=[];const start=now();let metadataArrival=NaN;
   try{await generate(origin,prompts[prompt]!,n,30000,f=>{frames.push(f);if(f.kind===0)metadataArrival=now();if(f.kind===1){arrivals.push(now());publication.push(Number(f.elapsedUs)/1000);tokens.push(f.tokenId);}});}finally{writeFileSync(dir+`/request-${ordinal}.amst`,Buffer.concat(frames.map(f=>encode(f))));}
   const end=now(),metadata=frames[0]!;if(!metadata.profile.includes(`width=${options.draftWidth} `)||!metadata.profile.includes(`sampling=${options.sampling} `)||!metadata.profile.includes(`expert_kernel=${options.expertKernel} `)||!metadata.profile.includes(`execution_timing=${options.executionTiming} `)||!metadata.profile.includes(`graph_mode=${options.graphMode} `)||metadata.modelIdentity!==refs.identity||!metadata.profile.includes(`execution=${profileLabels[profile]}`)||!metadata.profile.includes(`recurrent_kernel=${options.recurrentKernel} `)||!metadata.profile.includes(`layer_group=${options.layerGroup} `)||!metadata.profile.includes(`physical_tensor_batching=${profile==='target-physical'}`)||tokens.length!==n||tokens.some((t,i)=>t!==refs.tokens[prompt]![i]))throw Error('reference/profile/length mismatch');return {ordinal,prompt,start,end,arrivals,publication,tokens,metadata,metadataArrival};
  }
  const warmup=await request(-1,0,8);writeFileSync(dir+'/warmup.json',artifactJson(warmup));
  for(const lanes of [1,3]){
   const start=now();let delivered=0,ordinal=lanes*10000;const rows:Awaited<ReturnType<typeof request>>[]=[];
   while(delivered<options.tokens){const jobs=wavePlan(options.tokens-delivered,lanes).map(n=>{const id=ordinal++;return request(id,(id-lanes*10000)%prompts.length,n);});const completed=await Promise.all(jobs);rows.push(...completed);delivered+=completed.reduce((s,r)=>s+r.tokens.length,0);writeFileSync(dir+`/lanes-${lanes}-partial.json`,artifactJson(rows));}
   const end=now(),cell={profile,lanes,start,end,arrival_policy:'closed-loop synchronized waves; not continuous batching',...summarize(rows,start,end),requests:rows};results.push(cell);writeFileSync(dir+`/lanes-${lanes}.json`,artifactJson(cell));writeFileSync(out+'/partial.json',artifactJson(results));console.log(`CELL profile=${profile} lanes=${lanes} tokens=${delivered} tps=${cell.aggregate_tokens_per_second.toFixed(3)}`);
  }
  if(options.arrival==='closed+open'){
   const start=now();let active=0,highWater=0;
   const settled=await openLoop(16,250,start,{now,sleep:delay},async(i,scheduled,started)=>{
    active++;highWater=Math.max(highWater,active);
    try{const row=await request(90000+i,i%prompts.length,32);return {status:'success' as const,scheduled,started,end:row.end,row};}
    catch(error){return {status:error instanceof HttpStatusError&&error.status===503?'overloaded' as const:'failure' as const,scheduled,started,end:now(),error:String(error)};}
    finally{active--;}
   });
   const attempts=settled.map(r=>r.status==='fulfilled'?r.value:{status:'failure' as const,error:String(r.reason)}),end=now();writeFileSync(dir+'/open-attempts.json',artifactJson(attempts));
   if(attempts.some(r=>r.status==='failure'))throw Error('unexpected open-loop failure; every attempt settled and retained');
   const successful=attempts.flatMap(r=>r.status==='success'?[r.row]:[]),rejected=attempts.filter(r=>r.status==='overloaded').length;
   if(!successful.length||!rejected||active!==0)throw Error('open-loop pressure did not exercise successful admission and bounded rejection');
   const load={profile,start,end,offered_requests:16,offered_tokens:512,scheduled_interval_ms:250,scheduled_requests_per_second:4,scheduled_tokens_per_second:128,client_outstanding_high_water:highWater,successful_requests:successful.length,rejected_503:rejected,failed_requests:0,dispatch_jitter_ms:distribution(attempts.flatMap(r=>'started' in r?[r.started-r.scheduled]:[])),completion_latency_ms:distribution(attempts.flatMap(r=>'started' in r?[r.end-r.started]:[])),...summarize(successful,start,end),attempts};loads.push(load);writeFileSync(dir+'/open-loop.json',artifactJson(load));console.log(`OPEN profile=${profile} accepted=${successful.length} rejected=${rejected} tokens=${load.tokens}`);
  }
 }finally{
  if(!closed)child.kill('SIGTERM');if(!await waitForExit(exited,60000)){child.kill('SIGKILL');await exited;throw Error('owned HTTP process failed drain');}if(code!==0||!log.includes('native_bytes=0')||!log.includes('memory_used: 0'))throw Error('server drain failure');
 }
 const metrics=/BATCH_METRICS BatchCounters \{ packed_calls: (\d+), packed_rows: (\d+), scalar_calls: (\d+) \}/.exec(log);
 if(profile==='target'||profile==='target-physical'){
  if(!metrics)throw Error('missing target batch counters');
  const [packed_calls,packed_rows,scalar_calls]=metrics.slice(1).map(Number);
  if(profile==='target'?(packed_calls!==0||packed_rows!==0):(!packed_calls||packed_rows!<2*packed_calls!||packed_rows!>3*packed_calls!))throw Error('invalid packed-call coverage');
  writeFileSync(dir+'/packing.json',artifactJson({packed_calls,packed_rows,scalar_calls,scope:'successful adapter completions over warmup, closed cells and open arrivals; not per-cell or committed-token counters'}));
 }
 const ledger=/DRAFT_LEDGER proposed=(\d+) accepted=(\d+) known_seed=(\d+) committed=(\d+) target_calls=(\d+) failures=(\d+) unfinished=(\d+)/.exec(log);if(profile!=='target'&&profile!=='target-physical'&&!ledger)throw Error('missing proposer ledger');if(ledger){const [proposed,accepted,known_seed,committed,target_calls,failures,unfinished]=ledger.slice(1).map(Number);const baseline=/BASELINE_LEDGER committed=(\d+) cost_ns=(\d+)/.exec(log);if(!baseline)throw Error('missing no-draft baseline ledger');const baseline_committed=Number(baseline[1]),baseline_cost_ns=Number(baseline[2]);if(failures||unfinished)throw Error('failed/unfinished drafts');writeFileSync(dir+'/acceptance.json',artifactJson({proposed,accepted,known_seed,committed,target_calls,failures,unfinished,baseline_committed,baseline_cost_ns,fraction:proposed?accepted!/proposed!:null,scope:`cumulative warmup, one/three-lane cells and ${options.arrival==='closed+open'?'open-loop attempts':'no open-loop traffic'}; known seeds excluded from learned proposed/accepted`}));}
}
writeFileSync(out+'/summary.json',artifactJson({pass:true,scope:'explicit-order shared-host short-context HTTP controls; exact-reference tokens; not production or broad model qualification',options,total_closed_tokens:options.profiles.length*2*options.tokens,cells:results,open_loop:loads}));console.log(`PASS HTTP controls ${options.profiles.length*2*options.tokens} exact-reference closed-loop tokens`);
