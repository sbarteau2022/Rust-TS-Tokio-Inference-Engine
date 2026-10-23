"""Offline native pipe-disconnect gate; not a model worker or network transport."""
import os,select,struct,subprocess,sys,time
from pathlib import Path
out=Path(sys.argv[1]);out.mkdir()
command=['target/release/rust-server','native-mtp-stream','Complete: 1, 2, 3,','32','--draft-width','4','--stream-capacity','1','--qualification-trials','64','--probe-epochs','2','--initial-mass','2147483647','--admission-mass','268435455','--reward-mass','536870911']
(out/'command.txt').write_text(' '.join(command)+'\n')
with (out/'stderr.log').open('wb') as err:
 p=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=err)
 deadline=time.monotonic()+180
 def read_exact(n):
  result=b''
  while len(result)<n:
   remaining=deadline-time.monotonic()
   assert remaining>0,'native pipe read timeout'
   ready,_,_=select.select([p.stdout],[],[],remaining)
   assert ready,'native pipe read timeout'
   piece=os.read(p.stdout.fileno(),n-len(result))
   assert piece,'early EOF before disconnect test point'
   result+=piece
  return result
 try:
  captured=b''
  for _ in range(2):
   header=read_exact(4);size=struct.unpack('<I',header)[0];assert 12<=size<=72*1024
   captured+=header+read_exact(size)
  (out/'partial.amst').write_bytes(captured)
  p.stdout.close()
  code=p.wait(timeout=max(1,deadline-time.monotonic()))
 except BaseException:
  p.kill();p.wait();raise
text=(out/'stderr.log').read_text()
assert code!=0,'disconnected pipe must not succeed'
assert 'DRAIN scheduler=true native=true' in text,text
assert 'Broken pipe' in text,text
(out/'summary.txt').write_text(f'PASS disconnect after metadata + first token; exit={code}; scheduler/native drain verified; no HTTP claim\n')
print((out/'summary.txt').read_text(),end='')
