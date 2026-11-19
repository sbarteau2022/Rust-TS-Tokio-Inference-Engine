/** Offline benchmark controls and bounded arrival/lifetime primitives. */
export const profileLabels={target:'Target','target-physical':'Target',mtp:'Mtp','mtp-audited':'Audited'} as const;
export type Profile=keyof typeof profileLabels;
export function benchOptions(args:string[]){
 if(args.length<6||args.length>20||args.length%2)throw Error('required --profiles ORDER --arrival closed|closed+open --tokens-per-cell N; optional --layer-group, --recurrent-kernel, --execution-timing, --graph-mode');
 const values=new Map<string,string>();for(let i=0;i<args.length;i+=2){const key=args[i]!,value=args[i+1]!;if(!['--profiles','--arrival','--tokens-per-cell','--layer-group','--recurrent-kernel','--execution-timing','--graph-mode','--expert-kernel','--sampling','--draft-width'].includes(key)||values.has(key))throw Error('unknown/duplicate benchmark flag');values.set(key,value);}
 const profiles=values.get('--profiles')?.split(',')??[];if(!profiles.length||profiles.length>3||new Set(profiles).size!==profiles.length||profiles.some(p=>!Object.hasOwn(profileLabels,p)))throw Error('invalid profile order');
 const arrival=values.get('--arrival');if(arrival!=='closed'&&arrival!=='closed+open')throw Error('invalid arrival policy');
 const raw=values.get('--tokens-per-cell')??'';if(!/^\d+$/.test(raw))throw Error('decimal token budget required');const tokens=Number(raw);if(!Number.isSafeInteger(tokens)||tokens<1000||tokens>2000)throw Error('token budget 1000..2000');
 const group=values.get('--layer-group')??'1';if(!['1','4','10','40'].includes(group))throw Error('invalid layer group');
 const recurrent=values.get('--recurrent-kernel')??'reference';if(!['reference','fused'].includes(recurrent))throw Error('invalid recurrent kernel');
 const timing=values.get('--execution-timing')??'off';if(!['off','on'].includes(timing))throw Error('invalid execution timing');
 const graph=values.get('--graph-mode')??'eager';if(!['eager','compiled-decode'].includes(graph))throw Error('invalid graph mode');
 if(timing==='on'&&profiles.some(p=>p!=='target'&&p!=='target-physical'))throw Error('execution timing requires target profiles');
 const expert=values.get('--expert-kernel')??'reference';if(!['reference','sorted','fused'].includes(expert))throw Error('invalid expert kernel');
 const sampling=values.get('--sampling')??'host';if(!['host','device'].includes(sampling))throw Error('invalid sampling');
 if(sampling==='device'&&profiles.some(p=>p!=='target'))throw Error('device sampling requires scalar target');
 const width=values.get('--draft-width')??'4';if(!/^[1-8]$/.test(width))throw Error('draft width 1..8');if(profiles.some(p=>p==='mtp'||p==='mtp-audited')&&!['1','2','4','8'].includes(width))throw Error('HTTP MTP width must be 1|2|4|8');
 return {draftWidth:Number(width),profiles:profiles as Profile[],arrival,tokens,layerGroup:Number(group),recurrentKernel:recurrent,executionTiming:timing,graphMode:graph,expertKernel:expert,sampling};
}
export function wavePlan(left:number,lanes:number):number[]{if(!Number.isInteger(left)||left<1||left>2000||![1,3].includes(lanes))throw Error('wave bounds');const plan:number[]=[];for(let i=0;i<lanes&&left>0;i++){const n=Math.min(64,left);plan.push(n);left-=n;}return plan;}
export interface Clock {now():number;sleep(ms:number):Promise<unknown>}
export async function openLoop<T>(count:number,interval:number,origin:number,clock:Clock,attempt:(i:number,scheduled:number,started:number)=>Promise<T>){
 if(!Number.isInteger(count)||count<1||count>32||!Number.isInteger(interval)||interval<1||interval>1000||!Number.isFinite(origin))throw Error('arrival bounds');
 return Promise.allSettled(Array.from({length:count},async(_,i)=>{const scheduled=origin+i*interval;while(clock.now()<scheduled)await clock.sleep(Math.max(1,Math.ceil(scheduled-clock.now())));return attempt(i,scheduled,clock.now());}));
}
export async function waitForExit(exited:Promise<unknown>,ms:number):Promise<boolean>{
 if(!Number.isInteger(ms)||ms<1||ms>60000)throw Error('exit timeout bounds');let timer:ReturnType<typeof setTimeout>|undefined;
 try{return await Promise.race([exited.then(()=>true),new Promise<boolean>(resolve=>{timer=setTimeout(()=>resolve(false),ms);})]);}finally{clearTimeout(timer);}
}

/** One explicit native execution/memory mode for every benchmark label. */
export function executionFlags(profile:Profile):string[]{
 if(!Object.hasOwn(profileLabels,profile))throw Error('invalid profile');
 return ['--execution-profile',profile==='target-physical'?'target':profile,'--tensor-batching',profile==='target-physical'?'physical':'scalar'];
}
