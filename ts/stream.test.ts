import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { Decoder, decode, encode, frame, MAX_FRAME } from './stream.js';
const frames=[frame({modelIdentity:'a'.repeat(64),profile:'cross-language-experimental'}),frame({kind:1,sequence:9n,tokenId:42,text:'\ufeffé😀\n',elapsedUs:5n}),frame({kind:2,sequence:9n,outputTokens:1,stop:1,elapsedUs:8n,text:'�'})];
const concat=(xs:Uint8Array[])=>Buffer.concat(xs);
test('Rust fixture and all chunk sizes agree, BOM and Unicode preserved',()=>{
 const bytes=readFileSync('fixtures/stream-v1.amst');
 for(const input of [bytes,concat(frames.map(encode))])for(let chunk=1;chunk<=input.length;chunk++){const d=new Decoder();const got=[];for(let p=0;p<input.length;p+=chunk)got.push(...d.push(input.subarray(p,p+chunk)));d.finish();assert.deepEqual(got,frames);assert.throws(()=>d.push(new Uint8Array()));}
});
test('unknown controls, ordering, exact terminal and EOF refusal',()=>{
 for(const f of [frame({version:2}),frame({kind:255}),frame({...frames[1],text:'\ud800'}),frame({...frames[1],text:'x'.repeat(65537)})])assert.throws(()=>encode(f));
 for(const series of [[frames[1]!],[frames[0]!,frames[0]!],[frames[0]!,frame({...frames[1],index:1})],[...frames,frames[2]!],[frames[0]!,frames[2]!]]){const d=new Decoder();assert.throws(()=>d.push(concat(series.map(encode))));assert.throws(()=>d.push(new Uint8Array()));}
 const bytes=concat(frames.map(encode));for(let n=0;n<bytes.length;n++){const d=new Decoder();d.push(bytes.subarray(0,n));assert.throws(()=>d.finish());}
 for(const n of [0,11,MAX_FRAME+1,0xffffffff]){const b=new Uint8Array(4);new DataView(b.buffer).setUint32(0,n,true);assert.throws(()=>new Decoder().push(b));}
});
test('malformed structural mutations fail without permitting poisoned reuse',()=>{
 const bytes=concat(frames.map(encode));for(let p=0;p<bytes.length;p++)for(const value of [0,1,127,255]){const bad=Uint8Array.from(bytes);bad[p]=value;const d=new Decoder();try{d.push(bad);}catch{assert.throws(()=>d.push(new Uint8Array()));continue;}try{d.finish();}catch{/* visible terminal gap */}}
 const token=encode(frames[1]!);const marker=Buffer.from('é😀');const pos=Buffer.from(token).indexOf(marker);assert(pos>0);token[pos]=255;assert.throws(()=>decode(token));
});
test('typed errors terminate partial and empty streams',()=>{
 for(const seq of [0n,9n]){const series=seq===0n?[frames[0]!,frame({kind:3,errorCode:2,text:'deadline'})]:[frames[0]!,frames[1]!,frame({kind:3,sequence:9n,outputTokens:1,errorCode:4,text:'backend',elapsedUs:6n})];const d=new Decoder();assert.equal(d.push(concat(series.map(encode))).at(-1)?.kind,3);d.finish();}
});
test('timestamp, token and aggregate text caps fail closed',()=>{
 const d=new Decoder();d.push(concat(frames.slice(0,2).map(encode)));assert.throws(()=>d.push(encode(frame({...frames[2],elapsedUs:4n}))));
 const full=new Decoder();full.push(encode(frames[0]!));for(let i=0;i<128;i++)full.push(encode(frame({kind:1,sequence:9n,index:i,elapsedUs:BigInt(i)})));assert.throws(()=>full.push(encode(frames[1]!)));
 const text=new Decoder();text.push(encode(frames[0]!));for(let i=0;i<16;i++)text.push(encode(frame({kind:1,sequence:9n,index:i,text:'x'.repeat(65536)})));assert.throws(()=>text.push(encode(frame({kind:1,sequence:9n,index:16,text:'x'}))));
});
