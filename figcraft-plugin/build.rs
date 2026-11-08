use std::{env,fs,path::PathBuf};
fn a<'a>(t:&'a str,k:&str)->Option<&'a str>{let n=format!("{k}=\"");let s=t.find(&n)?+n.len();Some(&t[s..s+t[s..].find('"')?])}
fn n(t:&str,k:&str)->f64{a(t,k).and_then(|v|v.parse().ok()).unwrap_or(0.0)}
fn path(t:&str)->Option<String>{let k=t.split_whitespace().next()?.trim_start_matches('<').trim_end_matches('/');match k{
"path"=>a(t,"d").map(str::to_owned),
"line"=>Some(format!("M{} {}L{} {}",n(t,"x1"),n(t,"y1"),n(t,"x2"),n(t,"y2"))),
"rect"=>{let(x,y,w,h)=(n(t,"x"),n(t,"y"),n(t,"width"),n(t,"height"));if w<=0.||h<=0.{None}else{Some(format!("M{x} {y}h{w}v{h}H{x}z"))}},
"polyline"|"polygon"=>{let v:Vec<f64>=a(t,"points")?.replace(',',' ').split_whitespace().filter_map(|x|x.parse().ok()).collect();if v.len()<4||v.len()%2!=0{return None}let mut d=format!("M{} {}",v[0],v[1]);for p in v[2..].chunks(2){d.push_str(&format!("L{} {}",p[0],p[1]));}if k=="polygon"{d.push('z')}Some(d)},
"circle"|"ellipse"=>{let(cx,cy,rx,ry)=if k=="circle"{(n(t,"cx"),n(t,"cy"),n(t,"r"),n(t,"r"))}else{(n(t,"cx"),n(t,"cy"),n(t,"rx"),n(t,"ry"))};if rx<=0.||ry<=0.{None}else{Some(format!("M{} {}a{rx} {ry} 0 1 0 {} 0a{rx} {ry} 0 1 0 -{} 0",cx-rx,cy,rx*2.,rx*2.))}},
_=>None}}
fn main(){let root=PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("lucide-icons/icons");println!("cargo:rerun-if-changed={}",root.display());let mut fsx:Vec<_>=fs::read_dir(root).expect("Lucide checkout missing").filter_map(Result::ok).map(|e|e.path()).filter(|p|p.extension().is_some_and(|x|x=="svg")).collect();fsx.sort();let mut out=String::from("pub const ICONS: &[(&str,&str)] = &[\n");for f in fsx{let name=f.file_stem().unwrap().to_string_lossy().replace('_',"-");let svg=fs::read_to_string(f).unwrap();let mut rest=svg.as_str();let mut ps=Vec::new();while let Some(i)=rest.find('<'){rest=&rest[i..];let Some(j)=rest.find('>')else{break};if let Some(p)=path(&rest[..=j]){ps.push(p)}rest=&rest[j+1..]}if !ps.is_empty(){out.push_str(&format!("({:?},{:?}),\n",name,ps.join(" ")))}}out.push_str("];\n");fs::write(PathBuf::from(env::var("OUT_DIR").unwrap()).join("icons.rs"),out).unwrap();}
