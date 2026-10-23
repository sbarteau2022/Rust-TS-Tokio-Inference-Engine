/** Offline, fixed-order native HTTP pilot. No model execution takes place in JS. */
import {spawn,execFileSync} from 'node:child_process';import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';import {createHash} from 'node:crypto';import {createServer} from 'node:net';import {setTimeout as delay} from 'node:timers/promises';import {generate} from './http-client.js';import {summarize,artifactJson,type TimedRequest} from './http-metrics.js';import {encode,type Frame} from './stream.js';
const out=process.argv[2];if(!out||existsSync(out))throw Error('http-bench NEW_DIRECTORY');mkdirSync(out,{recursive:true});
const refs=JSON.parse(readFileSync('evidence/native-mtp-stream-matrix/raw/references.json','utf8')) as {identity:string;tokens:number[][]};const prompts=readFileSync('fixtures/native-speculative-prompts.tsv','utf8').trim().split('\n').map(s=>s.slice(s.indexOf('\t')+1));if(prompts.length!==refs.tokens.length)throw Error('reference count');
const now=()=>Number(process.hrtime.bigint())/1e6;const binary=new URL('../target/release/rust-server',import.meta.url).pathname;
writeFileSync(out+'/host.txt',execFileSync('sh',['-c','sw_vers; sysctl hw.memsize hw.ncpu; ps -axo pid,pcpu,rss,comm'],{encoding:'utf8'}));
const results:unknown[]=[];
for(const profile of ['target','mtp-audited']){
 const dir=out+'/'+profile;mkdirSync(dir);const listener=createServer();await new Promise<void>((ok,no)=>{listener.once('error',no);listener.listen(0,'127.0.0.1',ok)});const port=(listener.address() as {port:number}).port;await new Promise<void>(ok=>listener.close(()=>ok()));
 const args=['native-mtp-serve',`127.0.0.1:${port}`,'1000','8','--execution-profile',profile,'--draft-width','4','--stream-capacity','1','--qualification-trials','64','--probe-epochs','2','--initial-mass','2147483647','--admission-mass','268435455','--reward-mass','536870911'];
 writeFileSync(dir+'/command.json',artifactJson({binary,args,binary_sha256:createHash('sha256').update(readFileSync(binary)).digest('hex'),reference_identity:refs.identity}));
 const child=spawn(binary,args,{stdio:['ignore','pipe','pipe']});let log='',closed=false,code:number|null=null;const exited=new Promise<void>(ok=>child.on('close',c=>{closed=true;code=c;ok()}));for(const s of [child.stdout,child.stderr])s.on('data',b=>{log+=b.toString();writeFileSync(dir+'/server.log',log)});child.on('error',e=>{log+='\nSPAWN_ERROR '+String(e);writeFileSync(dir+'/server.log',log)});
 try{
  const until=now()+180000;while(!log.includes('READY http://')){if(closed)throw Error('server exited before ready');if(now()>until)throw Error('ready timeout; owned process still requires drain');await delay(100);}
  const origin=`http://127.0.0.1:${port}`;
  async function request(ordinal:number,prompt:number,n:number):Promise<TimedRequest&{ordinal:number;prompt:number;tokens:number[];metadata:Frame}>{
   const arrivals:number[]=[],publication:number[]=[],tokens:number[]=[],frames:Frame[]=[];const start=now();
   try{await generate(origin,prompts[prompt]!,n,30000,f=>{frames.push(f);if(f.kind===1){arrivals.push(now());publication.push(Number(f.elapsedUs)/1000);tokens.push(f.tokenId);}});}finally{writeFileSync(dir+`/request-${ordinal}.amst`,Buffer.concat(frames.map(f=>encode(f))));}
   const end=now(),metadata=frames[0]!;if(metadata.modelIdentity!==refs.identity||!metadata.profile.includes(`execution=${profile==='target'?'Target':'Audited'}`)||tokens.length!==n||tokens.some((t,i)=>t!==refs.tokens[prompt]![i]))throw Error('reference/profile/length mismatch');return {ordinal,prompt,start,end,arrivals,publication,tokens,metadata};
  }
  const warmup=await request(-1,0,8);writeFileSync(dir+'/warmup.json',artifactJson(warmup));
  for(const lanes of [1,3]){
   const start=now();let delivered=0,ordinal=lanes*10000;const rows:Awaited<ReturnType<typeof request>>[]=[];
   while(delivered<2000){const jobs:Promise<Awaited<ReturnType<typeof request>>>[]=[];let reserved=0;for(let lane=0;lane<lanes&&delivered+reserved<2000;lane++){const n=Math.min(64,2000-delivered-reserved),id=ordinal++;reserved+=n;jobs.push(request(id,(id-lanes*10000)%prompts.length,n));}const completed=await Promise.all(jobs);rows.push(...completed);delivered+=completed.reduce((s,r)=>s+r.tokens.length,0);writeFileSync(dir+`/lanes-${lanes}-partial.json`,artifactJson(rows));}
   const end=now(),cell={profile,lanes,arrival_policy:'closed-loop synchronized waves; not continuous batching',...summarize(rows,start,end),requests:rows};results.push(cell);writeFileSync(dir+`/lanes-${lanes}.json`,artifactJson(cell));writeFileSync(out+'/partial.json',artifactJson(results));console.log(`CELL profile=${profile} lanes=${lanes} tokens=${delivered} tps=${cell.aggregate_tokens_per_second.toFixed(3)}`);
  }
 }finally{
  if(!closed)child.kill('SIGTERM');const drained=await Promise.race([exited.then(()=>true),delay(60000).then(()=>false)]);if(!drained){child.kill('SIGKILL');await exited;throw Error('owned HTTP process failed drain');}if(code!==0||!log.includes('native_bytes=0')||!log.includes('memory_used: 0'))throw Error('server drain failure');
 }
 const ledger=/DRAFT_LEDGER proposed=(\d+) accepted=(\d+) known_seed=(\d+) committed=(\d+) target_calls=(\d+) failures=(\d+) unfinished=(\d+)/.exec(log);if(profile!=='target'&&!ledger)throw Error('missing proposer ledger');if(ledger){const [proposed,accepted,known_seed,committed,target_calls,failures,unfinished]=ledger.slice(1).map(Number);if(failures||unfinished)throw Error('failed/unfinished drafts');writeFileSync(dir+'/acceptance.json',artifactJson({proposed,accepted,known_seed,committed,target_calls,failures,unfinished,fraction:proposed?accepted!/proposed!:null,scope:'cumulative warmup plus one/three-lane cells; known seeds excluded from learned proposed/accepted'}));}
}
writeFileSync(out+'/summary.json',artifactJson({pass:true,scope:'fixed-order shared-host short-context HTTP pilot; exact-reference tokens; not production or broad model qualification',total_tokens:8000,cells:results}));console.log('PASS HTTP pilot 8000 exact-reference tokens');
