"""Independent analytic accuracy judge on saved raw integer GPU outputs."""
import json
import math
from pathlib import Path

root=Path(__file__).resolve().parents[1]
scale=1 << 31
results={}
for function in ['sin','cos','atan2','tanh']:
    directory=root/'artifacts'/function
    inputs=json.loads((directory/'inputs.json').read_text())
    actual=json.loads((directory/'result.json').read_text())['values']
    assert len(actual)==len(inputs['A'])
    max_raw_error=0
    for i,raw in enumerate(actual):
        assert type(raw) is int and -scale<=raw<scale
        a=inputs['A'][i]/scale
        if function=='sin': expected=math.sin(math.pi*a)
        elif function=='cos': expected=math.cos(math.pi*a)
        elif function=='tanh': expected=math.tanh(a)
        else:
            b=inputs['B'][i]/scale
            expected=0. if a==b==0 else math.atan2(a,b)/math.pi
        error=abs(raw/scale-expected)
        if function=='atan2': error=min(error,abs(2-error))
        max_raw_error=max(max_raw_error,error*scale)
        assert error*scale<=2.01,(function,i,raw,expected,error*scale)
    results[function]={'outputs':len(actual),'max_raw_error_vs_libm':max_raw_error,'allowed_raw_error':2.01}
print(json.dumps({'pass':True,'dtype':'q1_31_i32','accuracy':results,
                  'scope':'Independent approximate analytic accuracy; exact CPU/GPU integer parity is checked separately by the runtime. Not an exhaustive 2^32 input proof.'},indent=2))
