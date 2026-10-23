import * as fb from 'flatbuffers';
import { Frame as Wire } from './generated/stream-wire/frame.js';
export const MAX_FRAME=72*1024;
export interface Frame {version:number;kind:number;sequence:bigint;index:number;tokenId:number;elapsedUs:bigint;text:string;modelIdentity:string;profile:string;outputTokens:number;stop:number;errorCode:number}
export const frame=(v:Partial<Frame>):Frame=>({version:1,kind:0,sequence:0n,index:0,tokenId:0,elapsedUs:0n,text:'',modelIdentity:'',profile:'',outputTokens:0,stop:0,errorCode:0,...v});
function check(ok:unknown,message:string):asserts ok {if(!ok)throw Error(message);}
const length=(s:string)=>new TextEncoder().encode(s).length;
function validate(f:Frame){
 for(const s of [f.text,f.profile,f.modelIdentity])for(let i=0;i<s.length;i++){const c=s.charCodeAt(i);if(c>=0xd800&&c<=0xdbff){const next=s.charCodeAt(++i);check(next>=0xdc00&&next<=0xdfff,'unpaired UTF-16 surrogate');}else check(c<0xdc00||c>0xdfff,'unpaired UTF-16 surrogate');}
 for(const n of [f.version,f.kind,f.index,f.tokenId,f.outputTokens,f.stop,f.errorCode])check(Number.isSafeInteger(n)&&n>=0&&n<=0xffffffff,'integer bounds');
 check(f.sequence>=0n&&f.sequence<=0xffffffffffffffffn&&f.elapsedUs>=0n&&f.elapsedUs<=0xffffffffffffffffn,'u64 bounds');
 check(f.version===1&&length(f.text)<=65536&&length(f.profile)<=4096,'version/string bounds');
 const common=f.modelIdentity===''&&f.profile==='';
 switch(f.kind){
 case 0:check(/^[0-9a-f]{64}$/.test(f.modelIdentity)&&f.profile!==''&&f.sequence===0n&&f.index===0&&f.tokenId===0&&f.elapsedUs===0n&&f.text===''&&f.outputTokens===0&&f.stop===0&&f.errorCode===0,'metadata fields');break;
 case 1:check(common&&f.sequence>0n&&f.index<128&&f.tokenId<248320&&f.outputTokens===0&&f.stop===0&&f.errorCode===0,'token fields');break;
 case 2:check(common&&f.sequence>0n&&f.index===0&&f.tokenId===0&&f.outputTokens>=1&&f.outputTokens<=128&&(f.stop===1||f.stop===2)&&f.errorCode===0,'finish fields');break;
 case 3:check(common&&f.index===0&&f.tokenId===0&&f.outputTokens<=128&&f.stop===0&&f.errorCode>=1&&f.errorCode<=7&&f.text!==''&&length(f.text)<=4096,'error fields');break;
 default:throw Error('unknown kind');
 }
}
export function encode(f:Frame):Uint8Array{validate(f);const b=new fb.Builder();const t=b.createString(f.text),m=b.createString(f.modelIdentity),p=b.createString(f.profile);const root=Wire.createFrame(b,f.version,f.kind,f.sequence,f.index,f.tokenId,f.elapsedUs,t,m,p,f.outputTokens,f.stop,f.errorCode);Wire.finishSizePrefixedFrameBuffer(b,root);const out=b.asUint8Array();check(out.length<=MAX_FRAME+4,'frame byte cap');return out;}
// Generated accessors are used only after this independent, flat-table verifier.
function preflight(bytes:Uint8Array){
 check(bytes.length>=16&&bytes.length<=MAX_FRAME+4,'frame bounds');const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
 const range=(p:number,n:number)=>{check(Number.isSafeInteger(p)&&p>=0&&p+n<=bytes.length,'buffer bounds');};
 const u16=(p:number)=>{range(p,2);return v.getUint16(p,true);};const u32=(p:number)=>{range(p,4);return v.getUint32(p,true);};
 check(u32(0)===bytes.length-4,'length mismatch');check(bytes[8]===65&&bytes[9]===77&&bytes[10]===83&&bytes[11]===84,'identifier');
 const root=4+u32(4);range(root,4);check(root%4===0,'table alignment');const table=root-v.getInt32(root,true);range(table,4);check(table%2===0,'vtable alignment');const vt=u16(table),object=u16(table+2);check(vt>=4&&vt<=28&&vt%2===0&&object>=4,'vtable shape');range(table,vt);range(root,object);
 const widths=[2,1,8,4,4,8,4,4,4,4,1,4];const utf8=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true});
 for(let i=0;i<12;i++){if(4+i*2>=vt)break;const off=u16(table+4+i*2);if(!off)continue;const w=widths[i]!;check(off>=4&&off+w<=object,'field object bounds');const pos=root+off;check(pos%w===0,'field alignment');range(pos,w);
  if(i>=6&&i<=8){const relative=u32(pos);check(relative>=4,'string offset');const str=pos+relative,n=u32(str);check(str%4===0,'string alignment');check(n<=65536,'string cap');range(str+4,n+1);check(bytes[str+4+n]===0,'string terminator');utf8.decode(bytes.subarray(str+4,str+4+n));}
 }
}
function wireText(value:string|Uint8Array|null):string {check(value===null||value instanceof Uint8Array,'string bytes');return value===null?'':new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(value);}
export function decode(bytes:Uint8Array):Frame{preflight(bytes);const f=Wire.getSizePrefixedRootAsFrame(new fb.ByteBuffer(bytes));const out:Frame={version:f.version(),kind:f.kind(),sequence:f.sequence(),index:f.index(),tokenId:f.tokenId(),elapsedUs:f.elapsedUs(),text:wireText(f.text(fb.Encoding.UTF8_BYTES)),modelIdentity:wireText(f.modelIdentity(fb.Encoding.UTF8_BYTES)),profile:wireText(f.profile(fb.Encoding.UTF8_BYTES)),outputTokens:f.outputTokens(),stop:f.stop(),errorCode:f.errorCode()};validate(out);return out;}
export class Decoder {
 private pending=new Uint8Array(MAX_FRAME+4);private used=0;private target=4;private metadata=false;private terminal=false;private count=0;private sequence=0n;private last=0n;private totalText=0;private frames=0;private poisoned=false;
 push(input:Uint8Array):Frame[]{check(!this.poisoned,'parser poisoned');this.poisoned=true;const out:Frame[]=[];let at=0;
  while(at<input.length){check(!this.terminal,'bytes after terminal');const n=Math.min(this.target-this.used,input.length-at);this.pending.set(input.subarray(at,at+n),this.used);this.used+=n;at+=n;
   if(this.used===4&&this.target===4){const size=new DataView(this.pending.buffer).getUint32(0,true);check(size>=12&&size<=MAX_FRAME,'frame length cap');this.target=size+4;}
   if(this.used===this.target){const f=decode(this.pending.subarray(0,this.used));this.accept(f);out.push(f);this.used=0;this.target=4;}
  }this.poisoned=false;return out;
 }
 private accept(f:Frame){check(!this.terminal&&this.frames<130&&this.totalText+length(f.text)<=1024*1024,'stream limits');if(f.kind===0){check(!this.metadata&&this.frames===0,'metadata order');this.metadata=true;}else{check(this.metadata&&f.elapsedUs>=this.last,'metadata/time order');switch(f.kind){case 1:check(f.index===this.count&&this.count<128&&(this.sequence===0n||this.sequence===f.sequence),'token sequence/index');this.sequence=f.sequence;this.count++;break;case 2:check(f.sequence===this.sequence&&f.outputTokens===this.count&&this.count>0,'finish count/sequence');this.terminal=true;break;case 3:check(f.sequence===this.sequence&&f.outputTokens===this.count,'error count/sequence');this.terminal=true;break;}this.last=f.elapsedUs;}this.totalText+=length(f.text);this.frames++;}
 finish(){check(!this.poisoned,'parser poisoned');this.poisoned=true;check(this.used===0&&this.terminal,'unfinished/truncated stream');}
}
