"""Build the omitted-cast control in an isolated copy; canonical source stays fixed."""
import difflib,hashlib,json,shutil,subprocess,tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[2];out=root/'evidence/native-discriminators'
source=root/'native/qwen_model.inc';corrected=source.read_bytes();needle=b'if(c.activation_precision){embed_scales=mx::astype(embed_scales,mx::float32,s);embed_biases=mx::astype(embed_biases,mx::float32,s);}'
assert corrected.count(needle)==1
uncorrected=corrected.replace(needle,b'// Ablation only: retain BF16 embedding affine arithmetic.')
report={'canonical_source_sha256':hashlib.sha256(corrected).hexdigest(),'control_source_sha256':hashlib.sha256(uncorrected).hexdigest(),'canonical_source_modified':False,'variation':'omit only embedding scales/biases FP32 widening'}
(out/'embedding-ab-source.patch').write_text(''.join(difflib.unified_diff(corrected.decode().splitlines(True),uncorrected.decode().splitlines(True),fromfile='corrected/native/qwen_model.inc',tofile='control/native/qwen_model.inc')))
with tempfile.TemporaryDirectory(prefix='rust-server-embedding-ab-') as directory:
 tmp=Path(directory)
 for name in ['src','native','fixtures','examples']:shutil.copytree(root/name,tmp/name)
 for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,tmp/name)
 (tmp/'vendor').symlink_to(root/'vendor',target_is_directory=True)
 (tmp/'native/qwen_model.inc').write_bytes(uncorrected)
 (tmp/'target').mkdir()
 with (out/'embedding-ab-build.log').open('w') as log:
  clone=subprocess.run(['cp','-cR',str(root/'target/release'),str(tmp/'target/release')],stdout=log,stderr=subprocess.STDOUT)
  report['release_cache_cloned']=clone.returncode==0
  build=subprocess.run(['cargo','build','--release','--locked','--example','native_discriminators'],cwd=tmp,stdout=log,stderr=subprocess.STDOUT)
  assert build.returncode==0
 binary=tmp/'target/release/examples/native_discriminators';report['control_binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
 with (out/'native-uncorrected.log').open('w') as log:
  run=subprocess.run([str(binary)],cwd=tmp,stdout=log,stderr=subprocess.STDOUT)
  report['control_exit']=run.returncode;assert run.returncode==0
assert source.read_bytes()==corrected
(out/'embedding-ab-provenance.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS isolated embedding ablation; canonical source unchanged')
