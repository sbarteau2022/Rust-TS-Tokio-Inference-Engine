"""Three balanced-order repetitions; each process owns fresh warmed worker resources."""
import json, subprocess, pathlib, time, platform, hashlib
root = pathlib.Path(__file__).resolve().parents[1]
out = root/'evidence/macro-throughput.json'
rows=[]
configs=[(t,l,n) for t in [64,128,256,512] for l in [2,3] for n in [1000,1500,2000]]
started=time.time()
for repetition in range(3):
    order=configs if repetition != 1 else list(reversed(configs))
    for threads,limit,count in order:
        result=subprocess.run([str(root/'target/release/rust-server'),'macro-bench','artifacts',str(threads),str(limit),str(count)],cwd=root,text=True,capture_output=True,timeout=180,check=True)
        row=json.loads(result.stdout);row['repetition']=repetition+1;rows.append(row)
        out.write_text(json.dumps({'platform':platform.platform(),'started_unix':started,'binary_sha256':hashlib.sha256((root/'target/release/rust-server').read_bytes()).hexdigest(),'completed':len(rows)==72,'rows':rows},indent=2)+'\n')
        print(f'{len(rows)}/72: {threads}/{limit}/{count}: {row["requests_per_second"]:.1f} req/s',flush=True)
