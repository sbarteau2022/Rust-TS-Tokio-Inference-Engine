import {test} from 'node:test';import assert from 'node:assert/strict';import {spawnSync} from 'node:child_process';import {benchOptions,executionFlags,wavePlan,openLoop,waitForExit} from './http-bench-config.js';
const args=(s:string)=>s.split(' ');
test('benchmark flags preserve order and fail before side effects',()=>{const o=benchOptions(args('--profiles mtp-audited,mtp,target --arrival closed+open --tokens-per-cell 2000'));assert.deepEqual(o.profiles,['mtp-audited','mtp','target']);assert.equal(o.tokens,2000);for(const s of ['','--profiles target --arrival closed','--profiles target,target --arrival closed --tokens-per-cell 2000','--profiles bogus --arrival closed --tokens-per-cell 2000','--profiles target --arrival open --tokens-per-cell 2000','--profiles target --arrival closed --tokens-per-cell 999','--profiles target --arrival closed --tokens-per-cell 2001','--profiles target --arrival closed --tokens-per-cell 1e3','--profiles target --profiles mtp --tokens-per-cell 2000'])assert.throws(()=>benchOptions(args(s)));});
test('closed waves reserve exact final token budget',()=>{assert.deepEqual(wavePlan(80,3),[64,16]);assert.deepEqual(wavePlan(16,3),[16]);let left=2000,total=0;while(left){const wave=wavePlan(left,3);const n=wave.reduce((a,b)=>a+b,0);left-=n;total+=n;}assert.equal(total,2000);assert.throws(()=>wavePlan(1,4));assert.throws(()=>wavePlan(0,3));});
test('absolute arrivals proceed while earlier work is unfinished and retain failures',async()=>{let time=0;const sleepers:{at:number;done:()=>void}[]=[];const starts:number[]=[];let complete!:()=>void;const held=new Promise<void>(ok=>{complete=ok});const clock={now:()=>time,sleep:(ms:number)=>new Promise<void>(done=>sleepers.push({at:time+ms,done}))};const work=openLoop(3,250,0,clock,async(i,scheduled,started)=>{assert.equal(scheduled,started);starts.push(started);if(i===0)await held;if(i===1)throw Error('injected');return i;});assert.deepEqual(starts,[0]);for(const next of [250,500]){time=next;for(const s of sleepers.splice(0))if(s.at<=time)s.done();else sleepers.push(s);await Promise.resolve();}assert.deepEqual(starts,[0,250,500]);complete();const all=await work;assert.deepEqual(all.map(r=>r.status),['fulfilled','rejected','fulfilled']);});
test('shutdown timer is bounded and canceled on normal process exit',async()=>{assert.equal(await waitForExit(Promise.resolve(),60000),true);assert.equal(await waitForExit(new Promise(()=>{}),1),false);await assert.rejects(waitForExit(Promise.resolve(),0));const child=spawnSync(process.execPath,['--import','tsx','--input-type=module','-e',"import {waitForExit} from './ts/http-bench-config.ts'; await waitForExit(Promise.resolve(),60000);"],{timeout:3000,encoding:'utf8'});assert.equal(child.status,0,child.stderr||String(child.error));});

test('physical target benchmark labels select explicit server modes',()=>{
 assert.deepEqual(executionFlags('target'),['--execution-profile','target','--tensor-batching','scalar']);
 assert.deepEqual(executionFlags('target-physical'),['--execution-profile','target','--tensor-batching','physical']);
 assert.deepEqual(executionFlags('mtp-audited'),['--execution-profile','mtp-audited','--tensor-batching','scalar']);
 assert.deepEqual(benchOptions(args('--profiles target-physical,target --arrival closed+open --tokens-per-cell 2000')).profiles,['target-physical','target']);
 assert.throws(()=>executionFlags('bogus' as never));
});

test('layer grouping is explicit and refuses unsupported speculation before launch',()=>{
 const base='--profiles target-physical,target --arrival closed --tokens-per-cell 2000';
 assert.equal(benchOptions(args(base)).layerGroup,1);
 for(const g of [1,4,10,40])assert.equal(benchOptions(args(base+` --layer-group ${g}`)).layerGroup,g);
 for(const g of ['0','3','04','41','-1','1e1'])assert.throws(()=>benchOptions(args(base+` --layer-group ${g}`)));
 assert.throws(()=>benchOptions(args('--profiles mtp --arrival closed --tokens-per-cell 2000 --layer-group 4')));
 assert.throws(()=>benchOptions(args(base+' --layer-group 4 --layer-group 10')));
});

test('recurrent ablation validates mode and compatible execution profile',()=>{
 const base='--profiles target --arrival closed --tokens-per-cell 2000';
 assert.equal(benchOptions(args(base)).recurrentKernel,'reference');
 assert.equal(benchOptions(args(base+' --recurrent-kernel fused --layer-group 40')).recurrentKernel,'fused');
 assert.throws(()=>benchOptions(args(base+' --recurrent-kernel auto')));
 assert.throws(()=>benchOptions(args('--profiles mtp --arrival closed --tokens-per-cell 2000 --recurrent-kernel fused')));
});
