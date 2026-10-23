/** Native AMST HTTP client. Text exists only at the external request/response boundary. */
import { Decoder, type Frame } from './stream.js';
export async function generate(url:string,prompt:string,maxNew:number,deadlineMs:number,onFrame:(f:Frame)=>void,signal?:AbortSignal):Promise<void>{
 const u=new URL(url);if(u.protocol!=='http:'||!['127.0.0.1','[::1]'].includes(u.hostname)||u.pathname!=='/'||u.search||u.hash||u.username||u.password)throw Error('explicit loopback HTTP origin required');
 if(!Number.isInteger(maxNew)||maxNew<1||maxNew>128||!Number.isInteger(deadlineMs)||deadlineMs<1||deadlineMs>300000)throw Error('request bounds');
 const response=await fetch(new URL('/v1/generate',u),{method:'POST',headers:{'content-type':'text/plain; charset=utf-8','x-rust-max-new-tokens':String(maxNew),'x-rust-deadline-ms':String(deadlineMs)},body:prompt,signal});
 if(!response.ok){await response.body?.cancel();throw Error(`HTTP ${response.status}`);}
 if(response.headers.get('content-type')!=='application/vnd.rust-server.amst; version=1'||!response.body){await response.body?.cancel();throw Error('AMST response required');}
 const reader=response.body.getReader(),decoder=new Decoder();let terminal:number|undefined;
 try{for(;;){const part=await reader.read();if(part.done)break;for(const f of decoder.push(part.value)){terminal=f.kind;onFrame(f);}}decoder.finish();if(terminal!==2)throw Error('generation did not finish successfully');}finally{await reader.cancel().catch(()=>{});reader.releaseLock();}
}
