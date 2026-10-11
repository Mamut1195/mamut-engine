import fs from 'fs';
const [,, path, pre] = process.argv;
const { instance } = await WebAssembly.instantiate(fs.readFileSync(path), {});
const e = instance.exports;
const med = (f, k) => { f(); const t=[]; for (let i=0;i<k;i++){const s=performance.now(); f(); t.push(performance.now()-s);} t.sort((a,b)=>a-b); return t[t.length>>1]; };
const h = (x)=>BigInt.asUintN(64,x).toString(16).padStart(16,'0');
for (const [n,k] of [[50,20],[200,7],[1000,3]]) {
  const g = med(()=>e.gen_hash(n),k);
  const c = med(()=>e[`${pre}_chol_hash`](n),k) - g;
  const l = med(()=>e[`${pre}_lu_hash`](n),k) - g;
  console.log(`${pre} n=${n} chol_hash=${h(e[`${pre}_chol_hash`](n))} lu_hash=${h(e[`${pre}_lu_hash`](n))} chol_ms~${c.toFixed(2)} lu_ms~${l.toFixed(2)} gen_ms=${g.toFixed(2)} resid=${e[`${pre}_chol_resid`](n).toExponential(2)}`);
}
