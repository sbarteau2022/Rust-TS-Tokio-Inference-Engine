import {test} from 'node:test';
import assert from 'node:assert/strict';
import {generate} from './http-client.js';
import {encode,frame} from './stream.js';
test('HTTP client validates origin and bounds before network',async()=>{
 for(const url of ['https://127.0.0.1:1','http://example.com','http://127.0.0.1/x','http://user@127.0.0.1','http://127.0.0.1/?x'])await assert.rejects(generate(url,'hello',1,1,()=>{}));
 await assert.rejects(generate('http://127.0.0.1:1','hello',129,1,()=>{}));
});
test('HTTP client accepts fragmented binary finish and rejects error or truncated EOF',async()=>{
 const original=globalThis.fetch;const metadata=encode(frame({modelIdentity:'a'.repeat(64),profile:'test'})),token=encode(frame({kind:1,sequence:1n,tokenId:1,text:'é'}));
 try{for(const mode of ['finish','error','truncated']){
  const end=mode==='finish'?encode(frame({kind:2,sequence:1n,outputTokens:1,stop:1})):mode==='error'?encode(frame({kind:3,sequence:1n,outputTokens:1,errorCode:2,text:'deadline'})):new Uint8Array();
  globalThis.fetch=async(_input,init)=>{assert.equal(init?.body,'hello');assert.equal((init?.headers as Record<string,string>)['x-rust-deadline-ms'],'100');return new Response(new ReadableStream({start(c){for(const part of [metadata,token,end])for(const byte of part)c.enqueue(Uint8Array.of(byte));c.close();}}),{headers:{'content-type':'application/vnd.rust-server.amst; version=1'}});};
  const seen:number[]=[];const work=generate('http://127.0.0.1:8080','hello',1,100,f=>seen.push(f.kind));if(mode==='finish'){await work;assert.deepEqual(seen,[0,1,2]);}else await assert.rejects(work);
 }}finally{globalThis.fetch=original;}
});
