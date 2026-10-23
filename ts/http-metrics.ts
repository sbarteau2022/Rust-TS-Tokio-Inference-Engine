/** Offline qualification arithmetic; times are monotonic milliseconds at client callbacks. */
export interface TimedRequest {start:number;end:number;arrivals:number[];publication:number[]}
export function distribution(values:number[]){if(!values.length)return {count:0,mean:null,p50:null,p95:null,max:null};if(values.some(v=>!Number.isFinite(v)||v<0))throw Error('invalid timing');const a=[...values].sort((a,b)=>a-b);return {count:a.length,mean:a.reduce((s,v)=>s+v,0)/a.length,p50:a[Math.ceil(a.length*.5)-1]!,p95:a[Math.ceil(a.length*.95)-1]!,max:a.at(-1)!};}
export function summarize(rows:TimedRequest[],start:number,end:number){
 if(!rows.length||!Number.isFinite(start)||!Number.isFinite(end)||end<=start)throw Error('empty/invalid cell');
 const ttft:number[]=[],itl:number[]=[],native:number[]=[],times:number[]=[];let decodeTokens=0,decodeMs=0;
 for(const r of rows){if(!Number.isFinite(r.start)||!Number.isFinite(r.end)||!r.arrivals.length||r.publication.length!==r.arrivals.length||r.start<start||r.end>end||r.end<r.arrivals.at(-1)!||r.arrivals[0]!<r.start)throw Error('request bounds');ttft.push(r.arrivals[0]!-r.start);
  for(let i=0;i<r.arrivals.length;i++){const t=r.arrivals[i]!,p=r.publication[i]!;if(!Number.isFinite(t)||!Number.isFinite(p)||p<0||(i>0&&(t<r.arrivals[i-1]!||p<r.publication[i-1]!)))throw Error('nonmonotonic timing');times.push(t);if(i){itl.push(t-r.arrivals[i-1]!);native.push(p-r.publication[i-1]!);}}
  decodeTokens+=r.arrivals.length-1;decodeMs+=r.arrivals.at(-1)!-r.arrivals[0]!;
 }
 times.sort((a,b)=>a-b);return {tokens:times.length,wall_seconds:(end-start)/1000,aggregate_tokens_per_second:times.length*1000/(end-start),ttft_ms:distribution(ttft),client_callback_itl_ms:distribution(itl),native_publication_itl_ms:distribution(native),post_first_tokens_per_lane_second:decodeMs>0?decodeTokens*1000/decodeMs:null,windows:[1000,1500,2000].filter(n=>n<=times.length).map(n=>({tokens:n,elapsed_seconds:(times[n-1]!-start)/1000,tokens_per_second:times[n-1]!>start?n*1000/(times[n-1]!-start):null}))};
}

/** Preserve u64 frame metadata exactly in offline JSON artifacts. */
export function artifactJson(value:unknown):string{return JSON.stringify(value,(_key,v)=>typeof v==='bigint'?v.toString():v,2);}
