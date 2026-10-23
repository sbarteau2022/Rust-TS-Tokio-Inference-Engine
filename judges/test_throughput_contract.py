import copy,unittest
from physical_throughput_verify import control_contract,self_test
class Contract(unittest.TestCase):
 def test_contract(self):
  for profiles,single in [(['target','target-physical'],False),(['target-physical','target'],False),(['target-physical'],True),(['target'],True)]:
   n=len(profiles);s={'pass':True,'options':{'profiles':profiles,'arrival':'closed+open','tokens':2000,'layerGroup':10,'recurrentKernel':'fused'},'cells':[{}]*(2*n),'open_loop':[{}]*n,'total_closed_tokens':4000*n}
   control_contract(s,single)
   for field,value in [('cells',[]),('open_loop',[]),('total_closed_tokens',1),('pass',False)]:
    d=copy.deepcopy(s);d[field]=value
    with self.assertRaises(AssertionError):control_contract(d,single)
   with self.assertRaises(AssertionError):control_contract(s,not single)
   if single:
    for key in ['layerGroup','recurrentKernel']:
     d=copy.deepcopy(s);del d['options'][key]
     with self.assertRaises(AssertionError):control_contract(d,True)
 def test_packing_corruption(self):self_test()
if __name__=='__main__':unittest.main()
