//! Real std Vec/String operations with exact, bounded symbolic lengths.
fn domain(n:u8)->bool { n<=64 }
#[unsafe(no_mangle)]
pub fn phage_target(n:u8,x:u8)->bool {
    if !domain(n) { return true; }
    let n=usize::from(n);
    let mut v=Vec::with_capacity(n);
    for i in 0..n { v.push(x.wrapping_add(i as u8)); }
    v.len()==n && (n==0 || v[n-1]==x.wrapping_add((n-1) as u8))
}
#[unsafe(no_mangle)]
pub fn extended(n:u8,x:u8)->bool {
    if !domain(n) { return true; }
    let n=usize::from(n);let data=[x;64];let mut v=Vec::with_capacity(n);
    v.extend_from_slice(&data[..n]);v.as_slice()==&data[..n]
}
#[unsafe(no_mangle)]
pub fn copied(n:u8,x:u8)->bool {
    if !domain(n) { return true; }
    let n=usize::from(n);let data=[x;64];let v=data[..n].to_vec();
    v.len()==n && v.as_slice()==&data[..n]
}
#[unsafe(no_mangle)]
pub fn utf8(n:u8,x:u8)->bool {
    if !domain(n) { return true; }
    let n=usize::from(n);let data=[x;64];let v=data[..n].to_vec();
    match String::from_utf8(v) { Ok(s)=>s.len()==n,Err(e)=>e.into_bytes().len()==n }
}
#[unsafe(no_mangle)]
pub fn off_by_one(n:u8,x:u8)->bool {
    if n==0 || !domain(n) { return true; }
    let n=usize::from(n);let mut v=Vec::with_capacity(n);
    for _ in 0..n { v.push(x); }
    v[n]=x;true
}
#[unsafe(no_mangle)]
pub fn uncapped(n:u64)->bool {
    if n==0 { return true; }
    let v=Vec::<u8>::with_capacity(n as usize);
    v.capacity()>=n as usize
}
