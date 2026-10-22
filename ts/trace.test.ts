import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Builder,mvk } from './trace.js';
import * as fb from 'flatbuffers';
import { Trace } from './generated/wire/trace.js';
test('MVK records exact dependency order',()=>{
 const b=mvk(73,29,91);assert.deepEqual(b.ops.map(x=>x.kind),['load','load','load','dot','add','relu','store']);
 assert.deepEqual(b.ops[3],{kind:'dot',a:0,b:1,out:3});
 assert.deepEqual(b.ops[6],{kind:'store',a:5,name:'C'});
 const bytes=b.bytes();assert.equal(Buffer.from(bytes.subarray(4,8)).toString(),'AMVK');
 assert.equal(Trace.getRootAsTrace(new fb.ByteBuffer(bytes)).opsLength(),7);
 assert.equal(Trace.getRootAsTrace(new fb.ByteBuffer(bytes)).version(),3);
 assert.deepEqual(bytes,b.bytes());
});
test('invalid shapes and unsafe names rejected',()=>{
 for(const n of [0,-1,NaN,Infinity,0.5,4097,2**32])assert.throws(()=>new Builder().load('A',n,8));
 for(const name of ['', '../shader', 'a;evil', 'A'.repeat(65), 'a\0b'])assert.throws(()=>new Builder().load(name,8,8));
});
test('undefined and invalid handles rejected',()=>{
 const b=new Builder();b.load('A',1,1);
 for(const id of [-1,1,0.5,NaN,Infinity]){assert.throws(()=>b.relu(id));assert.throws(()=>b.dot(0,id));assert.throws(()=>b.store(id,'C'));}
 assert.throws(()=>b.store(0,'bad/name'));
});

test('second kernel family records row-bias fusion',async()=>{
 const {addRelu}=await import('./trace.js');const b=addRelu(73,91);
 assert.deepEqual(b.ops.map(x=>x.kind),['load','load','add','relu','store']);
 assert.deepEqual(b.ops[1],{kind:'load',name:'bias',rows:1,cols:91,out:1});
 assert.equal(Trace.getRootAsTrace(new fb.ByteBuffer(b.bytes())).opsLength(),5);
});

test('CORDIC functions serialize stable version-3 operation codes',async()=>{
 const {cordic}=await import('./trace.js');
 for(const [fn,code] of [['sin',0],['cos',1],['atan2',2],['tanh',3]] as const){
  const b=cordic(fn,7,13);const root=Trace.getRootAsTrace(new fb.ByteBuffer(b.bytes()));
  assert.equal(root.version(),3);assert.equal(root.opsLength(),fn==='atan2'?4:3);
  const op=b.ops.at(-2);assert.equal(op?.kind,'cordic');
  assert.deepEqual(op,fn==='atan2'?{kind:'cordic',function:code,a:0,b:1,out:2}:{kind:'cordic',function:code,a:0,b:0,out:1});
 }
 const b=new Builder();const a=b.load('A',1,1),other=b.load('B',1,1);
 assert.throws(()=>b.cordic('sin',a,other));
 assert.throws(()=>b.cordic('atan2',a,99));
 // Deliberate runtime misuse bypasses TS's compile-time function union.
 assert.throws(()=>b.cordic('bad' as 'sin',a));
});
