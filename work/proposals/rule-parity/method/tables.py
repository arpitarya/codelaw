import csv,json,collections,os,sys
M=os.path.dirname(os.path.abspath(__file__))+'/'; P=os.path.dirname(M[:-1])+'/'
s=json.load(open(P+'summary.json'))
CL=['covered','port','types','layout','planned','needs-data','out-of-scope','same-as','skip','retired']
print('TOOLTABLE'); print('| tool | version | rules | '+' | '.join(CL)+' | applicable | codelaw covers |'); print('|---|---|--:|'+'--:|'*len(CL)+'--:|--:|')
T=collections.Counter()
for t in sorted(s, key=lambda k:-s[k]['total']):
    if t=='prettier': continue
    x=s[t]; app=x['total']-sum(x.get(k,0) for k in ['retired','layout','out-of-scope','same-as'])
    pct=f"{100*x.get('covered',0)/app:.0f}%" if app else '—'
    print(f"| {t} | {x['version']} | {x['total']} | "+' | '.join(str(x.get(k,0) or '') for k in CL)+f" | {app} | {pct} |")
    for k in CL: T[k]+=x.get(k,0)
    T['total']+=x['total']; T['app']+=app
print(f"| **all** | | **{T['total']}** | "+' | '.join(f"**{T[k]}**" for k in CL)+f" | **{T['app']}** | **{100*T['covered']/T['app']:.0f}%** |")
allr=[]
for f in sorted(os.listdir(P)):
    if f.endswith('.csv') and f not in ('prettier.csv','tslint-recheck.csv','oxlint.csv'):
        for r in csv.DictReader(open(P+f)): r['tool']=f[:-4]; allr.append(r)
ports=[r for r in allr if r['class']=='port']
def tranche(r):
    if r['priority']=='P3': return 'E'
    if r['tool'] in ('ruff','pylint'): return 'A' if r['priority']=='P1' else 'D'
    if r['tool']=='eslint-plugin-unicorn': return 'B'
    return 'A' if r['priority']=='P1' else 'C'
cl=json.load(open(M+'codelaw.json')); now=collections.Counter(r[0] for r in cl)
pp=collections.defaultdict(collections.Counter)
for r in ports: pp[r['proposed_pack']][tranche(r)]+=1
print('PACKTABLE'); print('| pack | codelaw today | + A | + B | + C | + D | + E | after all |'); print('|---|--:|--:|--:|--:|--:|--:|--:|')
tot=collections.Counter()
for pk in sorted(set(now)|set(pp)):
    a=now.get(pk,0); p=pp.get(pk,{})
    vals=[p.get(k,0) for k in 'ABCDE']; tot['now']+=a
    for k,v in zip('ABCDE',vals): tot[k]+=v
    print(f"| {pk} | {a} | "+' | '.join(str(v or '') for v in vals)+f" | {a+sum(vals)} |")
print(f"| **all** | **{tot['now']}** | "+' | '.join(f"**{tot[k]}**" for k in 'ABCDE')+f" | **{tot['now']+sum(tot[k] for k in 'ABCDE')}** |")
TT=collections.defaultdict(collections.Counter)
for r in ports: TT[tranche(r)][r['tool']]+=1
print('TRANCHES')
for k in 'ABCDE': print(k, sum(TT[k].values()), ', '.join(f'{t} {n}' for t,n in TT[k].most_common()))
fx=collections.Counter(r['proposed_fix'] for r in ports); print('FIX', dict(fx))
print('UNLINKED', sum(1 for r in allr if r['codelaw'].endswith('*')))
A=[r for r in ports if tranche(r)=='A']
print('ALIST')
for t in sorted({r['tool'] for r in A}): print(f"- **{t}** ({sum(1 for r in A if r['tool']==t)}): "+', '.join(f"`{r['upstream_id']}`" for r in A if r['tool']==t))
# tranche column into the CSVs, so W-139 can filter on it
for f in sorted(os.listdir(P)):
    if not f.endswith('.csv') or f in ('prettier.csv','tslint-recheck.csv','oxlint.csv'): continue
    rs=list(csv.DictReader(open(P+f)))
    if not rs: continue
    cols=list(rs[0].keys())
    if 'tranche' not in cols: cols.insert(cols.index('priority')+1,'tranche')
    for r in rs: r['tool']=f[:-4]; r['tranche']=tranche(r) if r['class']=='port' else ''
    w=csv.DictWriter(open(P+f,'w',newline=''),fieldnames=cols,extrasaction='ignore',lineterminator='\n'); w.writeheader(); w.writerows(rs)
