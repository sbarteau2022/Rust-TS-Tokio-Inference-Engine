import { openSync,readSync,closeSync,writeFileSync } from 'node:fs';
import { Decoder, encode, frame } from './stream.js';
if(process.argv[2]==='--fixture'){
 const path=process.argv[3];if(!path)throw Error('--fixture NEW_FILE');
 writeFileSync(path,Buffer.concat([encode(frame({modelIdentity:'a'.repeat(64),profile:'cross-language-experimental'})),encode(frame({kind:1,sequence:9n,tokenId:42,text:'\ufeffé😀\n',elapsedUs:5n})),encode(frame({kind:2,sequence:9n,outputTokens:1,stop:1,elapsedUs:8n,text:'�'}))]),{flag:'wx'});
}else{
 const path=process.argv[2];if(!path)throw Error('stream-inspect FILE');const d=new Decoder();const frames=[];const fd=openSync(path,'r');const chunk=new Uint8Array(4096);let total=0;try{for(;;){const n=readSync(fd,chunk,0,chunk.length,null);if(!n)break;total+=n;if(total>130*(72*1024+4))throw Error('stream file cap');frames.push(...d.push(chunk.subarray(0,n)));}}finally{closeSync(fd);}d.finish();const tokens=frames.filter(f=>f.kind===1);console.log(`STREAM frames=${frames.length} tokens=${tokens.length} terminal=${frames.at(-1)?.kind}`);process.stdout.write(frames.filter(f=>f.kind===1||f.kind===2).map(f=>f.text).join(''));
}
