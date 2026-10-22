import * as fb from 'flatbuffers';
import { Load } from './generated/wire/load.js';
import { Dot } from './generated/wire/dot.js';
import { Add } from './generated/wire/add.js';
import { Relu } from './generated/wire/relu.js';
import { Cordic } from './generated/wire/cordic.js';
import { Store } from './generated/wire/store.js';
import { Op as WireOp } from './generated/wire/op.js';
import { Trace } from './generated/wire/trace.js';
import { Operation } from './generated/wire/operation.js';
import { mkdirSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export type CordicFunction = 'sin'|'cos'|'atan2'|'tanh';
const functionCode = {sin:0,cos:1,atan2:2,tanh:3} as const;
export type Op = {kind:'load';name:string;rows:number;cols:number;out:number} |
 {kind:'dot'|'add';a:number;b:number;out:number} | {kind:'relu';a:number;out:number} |
 {kind:'cordic';function:number;a:number;b:number;out:number} | {kind:'store';a:number;name:string};
export class Builder {
 readonly ops: Op[] = [];
 private next = 0;
 load(name:string, rows:number, cols:number):number {
  if (!/^[A-Za-z][A-Za-z0-9_]{0,63}$/.test(name) || ![rows,cols].every(x=>Number.isSafeInteger(x)&&x>0&&x<=4096)) throw Error('invalid load');
  const out=this.next++; this.ops.push({kind:'load',name,rows,cols,out}); return out;
 }
 dot(a:number,b:number):number {return this.binary('dot',a,b);}
 add(a:number,b:number):number {return this.binary('add',a,b);}
 cordic(fn:CordicFunction,a:number,b:number=a):number {
  if(!Object.hasOwn(functionCode,fn))throw Error('unsupported CORDIC function');
  this.use(a);this.use(b);if(fn!=='atan2'&&a!==b)throw Error('unary CORDIC second input');
  const out=this.next++;this.ops.push({kind:'cordic',function:functionCode[fn],a,b,out});return out;
 }
 private binary(kind:'dot'|'add',a:number,b:number):number {
  this.use(a);this.use(b);const out=this.next++;this.ops.push({kind,a,b,out});return out;
 }
 relu(a:number):number {this.use(a);const out=this.next++;this.ops.push({kind:'relu',a,out});return out;}
 store(a:number,name:string):void {this.use(a);if(!/^[A-Za-z][A-Za-z0-9_]{0,63}$/.test(name))throw Error('invalid store');this.ops.push({kind:'store',a,name});}
 private use(a:number):void {if(!Number.isSafeInteger(a)||a<0||a>=this.next)throw Error('undefined value');}
 bytes():Uint8Array {
  const b=new fb.Builder(1024);
  const offsets=this.ops.map(op=>{
   let body:number,type:Operation;
   switch(op.kind) {
    case 'load':body=Load.createLoad(b,b.createString(op.name),op.rows,op.cols,op.out);type=Operation.Load;break;
    case 'dot':body=Dot.createDot(b,op.a,op.b,op.out);type=Operation.Dot;break;
    case 'add':body=Add.createAdd(b,op.a,op.b,op.out);type=Operation.Add;break;
    case 'relu':body=Relu.createRelu(b,op.a,op.out);type=Operation.Relu;break;
    case 'cordic':body=Cordic.createCordic(b,op.function,op.a,op.b,op.out);type=Operation.Cordic;break;
    case 'store':body=Store.createStore(b,op.a,b.createString(op.name));type=Operation.Store;break;
   }
   return WireOp.createOp(b,type,body);
  });
  const vec=Trace.createOpsVector(b,offsets);const root=Trace.createTrace(b,3,vec);Trace.finishTraceBuffer(b,root);return b.asUint8Array();
 }
}
export function mvk(m:number,k:number,n:number):Builder {
 const b=new Builder();const a=b.load('A',m,k),w=b.load('B',k,n),bias=b.load('bias',1,n);
 b.store(b.relu(b.add(b.dot(a,w),bias)),'C');return b;
}
export function addRelu(m:number,n:number):Builder {
 const b=new Builder();const a=b.load('A',m,n),bias=b.load('bias',1,n);b.store(b.relu(b.add(a,bias)),'C');return b;
}
export function cordic(fn:CordicFunction,m:number,n:number):Builder {
 const b=new Builder();const a=b.load('A',m,n);const other=fn==='atan2'?b.load('B',m,n):a;
 b.store(b.cordic(fn,a,other),'C');return b;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const [dir='artifacts',ms='128',ks='128',ns='128',family='gemm']=process.argv.slice(2);
 if(!['gemm','add-relu','sin','cos','atan2','tanh'].includes(family))throw Error('unsupported family');
 const b=family==='gemm'?mvk(Number(ms),Number(ks),Number(ns)):family==='add-relu'?addRelu(Number(ms),Number(ns)):cordic(family as CordicFunction,Number(ms),Number(ns));mkdirSync(dir,{recursive:true});
 writeFileSync(`${dir}/trace.amvk`,b.bytes());writeFileSync(`${dir}/ops.json`,JSON.stringify(b.ops,null,2));
 console.log(JSON.stringify(b.ops));
}
