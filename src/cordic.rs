//! Integer-only CORDIC; mathematical accuracy and CPU/GPU bit parity are distinct gates.
use crate::{
    cordic_tables::{ATANH, ATAN_PI, CIRCULAR_GAIN},
    ir::KernelKind,
};
const ONE: i64 = 1 << 60;
const LIFT: i64 = 1 << 29;
fn clamp(x: i64) -> i32 {
    x.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
fn circular(angle: i32, cosine: bool) -> i32 {
    match angle {
        0 => return if cosine { i32::MAX } else { 0 },
        i32::MIN => return if cosine { i32::MIN } else { 0 },
        1073741824 => return if cosine { 0 } else { i32::MAX },
        -1073741824 => return if cosine { 0 } else { i32::MIN },
        _ => (),
    }
    let mut z = i64::from(angle) * LIFT;
    let mut sign = 1;
    if z > ONE / 2 {
        z -= ONE;
        sign = -1;
    }
    if z < -ONE / 2 {
        z += ONE;
        sign = -1;
    }
    let (mut x, mut y) = (CIRCULAR_GAIN, 0i64);
    for (i, step) in ATAN_PI.iter().enumerate() {
        let (xs, ys) = (x >> i, y >> i);
        if z >= 0 {
            x -= ys;
            y += xs;
            z -= step;
        } else {
            x += ys;
            y -= xs;
            z += step;
        }
    }
    clamp((sign * if cosine { x } else { y }) >> 29)
}
fn atan2(y_raw: i32, x_raw: i32) -> i32 {
    if y_raw == 0 {
        return if x_raw < 0 { i32::MIN } else { 0 };
    }
    if x_raw == 0 {
        return if y_raw > 0 { 1 << 30 } else { -(1 << 30) };
    }
    let (mut x, mut y, mut z) = (i64::from(x_raw) * LIFT, i64::from(y_raw) * LIFT, 0i64);
    while x.abs().max(y.abs()) < ONE / 2 {
        x *= 2;
        y *= 2;
    }
    if x < 0 {
        z = if y > 0 { ONE } else { -ONE };
        x = -x;
        y = -y;
    }
    for (i, step) in ATAN_PI.iter().enumerate() {
        let (xs, ys) = (x >> i, y >> i);
        if y > 0 {
            x += ys;
            y -= xs;
            z += step;
        } else if y < 0 {
            x -= ys;
            y += xs;
            z -= step;
        }
    }
    if z >= ONE {
        z -= 2 * ONE;
    }
    if z < -ONE {
        z += 2 * ONE;
    }
    clamp(z >> 29)
}
fn tanh(raw: i32) -> i32 {
    if raw == 0 {
        return 0;
    }
    let (mut x, mut y, mut z) = (ONE, 0i64, i64::from(raw) * LIFT);
    for (i, step) in ATANH.iter().enumerate().skip(1) {
        for _ in 0..if [4, 13, 40].contains(&i) { 2 } else { 1 } {
            let (xs, ys) = (x >> i, y >> i);
            if z >= 0 {
                x += ys;
                y += xs;
                z -= step;
            } else {
                x -= ys;
                y -= xs;
                z += step;
            }
        }
    }
    let mut remainder = y.abs();
    let mut quotient = 0i64;
    for _ in 0..31 {
        remainder *= 2;
        quotient *= 2;
        if remainder >= x {
            remainder -= x;
            quotient += 1;
        }
    }
    clamp(if y < 0 { -quotient } else { quotient })
}
pub fn evaluate(kind: KernelKind, a: i32, b: i32) -> i32 {
    match kind {
        KernelKind::Sin => circular(a, false),
        KernelKind::Cos => circular(a, true),
        KernelKind::Atan2 => atan2(a, b),
        KernelKind::Tanh => tanh(a),
        _ => unreachable!("CORDIC kind required"),
    }
}
pub fn msl(kind: KernelKind, count: u32) -> String {
    fn table(name: &str, values: &[i64]) -> String {
        format!(
            "constant int64_t {name}[48] = {{{}}};\n",
            values
                .iter()
                .map(|v| format!("{v}L"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
    let prefix = format!("#include <metal_stdlib>\nusing namespace metal;\n{}{}constant int64_t GAIN={CIRCULAR_GAIN}L;\n", table("ANG", &ATAN_PI), table("HYP", &ATANH));
    let body = r#"
constant int64_t ONE=1152921504606846976L;
constant int64_t LIFT=536870912L;
inline int64_t shift_floor(int64_t x, uint s) {
 if(s==0) return x;
 uint64_t bits=as_type<uint64_t>(x);
 return as_type<int64_t>((bits>>s)|(x<0 ? (~uint64_t(0)<<(64-s)) : uint64_t(0)));
}
inline int32_t sat(int64_t x) { return int32_t(clamp(x,-2147483648L,2147483647L)); }
inline int32_t circular(int32_t angle, bool cosine) {
 if(angle==0) return cosine?2147483647:0;
 if(angle==(-2147483647-1)) return cosine?(-2147483647-1):0;
 if(angle==1073741824) return cosine?0:2147483647;
 if(angle==-1073741824) return cosine?0:(-2147483647-1);
 int64_t z=int64_t(angle)*LIFT, x=GAIN, y=0, sign=1;
 if(z>ONE/2) {z-=ONE;sign=-1;}
 if(z<-ONE/2) {z+=ONE;sign=-1;}
 for(uint i=0;i<48;++i) {
  int64_t xs=shift_floor(x,i),ys=shift_floor(y,i);
  if(z>=0) {x-=ys;y+=xs;z-=ANG[i];}
  else {x+=ys;y-=xs;z+=ANG[i];}
 }
 return sat(shift_floor(sign*(cosine?x:y),29));
}
inline int32_t vector_angle(int32_t a, int32_t b) {
 if(a==0) return b<0?(-2147483647-1):0;
 if(b==0) return a>0?1073741824:-1073741824;
 int64_t x=int64_t(b)*LIFT,y=int64_t(a)*LIFT,z=0;
 while(max(x<0?-x:x,y<0?-y:y)<ONE/2) {x*=2;y*=2;}
 if(x<0) {z=y>0?ONE:-ONE;x=-x;y=-y;}
 for(uint i=0;i<48;++i) {
  int64_t xs=shift_floor(x,i),ys=shift_floor(y,i);
  if(y>0) {x+=ys;y-=xs;z+=ANG[i];}
  else if(y<0) {x-=ys;y+=xs;z-=ANG[i];}
 }
 if(z>=ONE) z-=2*ONE;
 if(z<-ONE) z+=2*ONE;
 return sat(shift_floor(z,29));
}
inline int32_t hyperbolic(int32_t a) {
 if(a==0) return 0;
 int64_t x=ONE,y=0,z=int64_t(a)*LIFT;
 for(uint i=1;i<48;++i) {
  uint repeats=(i==4||i==13||i==40)?2:1;
  for(uint repeat=0;repeat<repeats;++repeat) {
   int64_t xs=shift_floor(x,i),ys=shift_floor(y,i);
   if(z>=0) {x+=ys;y+=xs;z-=HYP[i];}
   else {x-=ys;y-=xs;z+=HYP[i];}
  }
 }
 int64_t remainder=y<0?-y:y,quotient=0;
 for(uint i=0;i<31;++i) {
  remainder*=2;quotient*=2;
  if(remainder>=x) {remainder-=x;quotient+=1;}
 }
 return sat(y<0?-quotient:quotient);
}
"#;
    let expression = match kind {
        KernelKind::Sin => "circular(A[i],false)",
        KernelKind::Cos => "circular(A[i],true)",
        KernelKind::Atan2 => "vector_angle(A[i],B[i])",
        KernelKind::Tanh => "hyperbolic(A[i])",
        _ => unreachable!("CORDIC kind required"),
    };
    let args = if kind == KernelKind::Atan2 {
        "device const int32_t* A [[buffer(0)]],device const int32_t* B [[buffer(1)]],device int32_t* C [[buffer(2)]]"
    } else {
        "device const int32_t* A [[buffer(0)]],device int32_t* C [[buffer(1)]]"
    };
    format!("{prefix}{body}\nkernel void {}({args},uint i [[thread_position_in_grid]]) {{if(i<{count}) C[i]={expression};}}",kind.entry())
}
