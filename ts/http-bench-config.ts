/** Offline benchmark controls and bounded arrival/lifetime primitives. */
export const profileLabels={target:'Target','target-physical':'Target',mtp:'Mtp','mtp-audited':'Audited'} as const;
export type Profile=keyof typeof profileLabels;
export function benchOptions(args:string[]){
 if(args.length!==6&&args.length!==8&&args.length!==10)throw Error('required --profiles ORDER --arrival closed|closed+open --tokens-per-cell N [--layer-group 1|4|10|40] [--recurrent-kernel reference|fused]');
 const values=new Map<string,string>();for(let i=0;i<args.length;i+=2){const key=args[i]!,value=args[i+1]!;if(!['--profiles','--arrival','--tokens-per-cell','--layer-group','--recurrent-kernel'].includes(key)||values.has(key))throw Error('unknown/duplicate benchmark flag');values.set(key,value);}
 const profiles=values.get('--profiles')?.split(',')??[];if(!profiles.length||profiles.length>3||new Set(profiles).size!==profiles.length||profiles.some(p=>!Object.hasOwn(profileLabels,p)))throw Error('invalid profile order');
 const arrival=values.get('--arrival');if(arrival!=='closed'&&arrival!=='closed+open')throw Error('invalid arrival policy');
 const raw=values.get('--tokens-per-cell')??'';if(!/^\d+$/.test(raw))throw Error('decimal token budget required');const tokens=Number(raw);if(!Number.isSafeInteger(tokens)||tokens<1000||tokens>2000)throw Error('token budget 1000..2000');
 const group=values.get('--layer-group')??'1';if(!['1','4','10','40'].includes(group))throw Error('invalid layer group');
 const recurrent=values.get('--recurrent-kernel')??'reference';if(!['reference','fused'].includes(recurrent))throw Error('invalid recurrent kernel');
 if((group!=='1'||recurrent==='fused')&&profiles.some(p=>p!=='target'&&p!=='target-physical'))throw Error('grouped layers require target profiles');
 return {profiles:profiles as Profile[],arrival,tokens,layerGroup:Number(group),recurrentKernel:recurrent};
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
