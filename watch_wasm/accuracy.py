
# import os
# import json
# import pandas as pd



# def true_positives(T, X, margin=5):
#     X = set(X)
#     TP = set()
#     for tau in T:
#         close = [(abs(tau - x), x) for x in X if abs(tau - x) <= margin]
#         if not close:
#             continue
#         dist, xstar = min(close)
#         TP.add(tau)
#         X.remove(xstar)
#     return TP

# def f_measure(annotations, predictions, margin=5, alpha=0.5, return_PR=False):
#     Tks = {k + 1: set(annotations[uid]) for k, uid in enumerate(annotations)}
#     for Tk in Tks.values():
#         Tk.add(0)
#     X = set(predictions)
#     X.add(0)
   
#     Tstar = set().union(*Tks.values())

#     P = len(true_positives(Tstar, X, margin=margin)) / len(X)

#     TPk = {k: true_positives(Tks[k], X, margin=margin) for k in Tks}
#     R = sum(len(TPk[k]) / len(Tks[k]) for k in Tks) / len(Tks)

#     denom = alpha * R + (1 - alpha) * P
#     F = (P * R / denom) if denom > 0 else 0.0

#     if return_PR:
#         return F, P, R
#     return F

# def overlap(A, B):
#     return len(A & B) / len(A | B) if A or B else 0.0

# def partition_from_cps(locations, n_obs):
#     """Turn a sorted list of change-points into segments (sets of indices)."""
#     cps = sorted(set(locations))
#     partition = []
#     current = set()
#     it = iter(cps)
#     try:
#         cp = next(it)
#     except StopIteration:
#         cp = None

#     for i in range(n_obs):
#         if i == cp:
#             if current:
#                 partition.append(current)
#             current = set()
#             try:
#                 cp = next(it)
#             except StopIteration:
#                 cp = None
#         current.add(i)

#     partition.append(current)
#     return partition

# def cover_single(S, Sprime):
#     """Covering score for one reference partition S vs. predicted Sprime."""
#     T = sum(len(r) for r in Sprime)
#     C = 0
#     for R in S:
#         C += len(R) * max(overlap(R, Rp) for Rp in Sprime)
#     return C / T if T > 0 else 0.0

# def covering(annotations, predictions, n_obs):
    
#     Ak = {
#         k + 1: partition_from_cps(annotations[uid], n_obs)
#         for k, uid in enumerate(annotations)
#     }
    
#     pX = partition_from_cps(predictions, n_obs)

    
#     scores = [cover_single(Ak[k], pX) for k in Ak]
#     return sum(scores) / len(scores) if scores else 0.0

# def clean_cps(locations, n_obs):
#     """Throw away any cp < 1 or ≥ n_obs−1."""
#     return sorted(cp for cp in locations if 1 <= cp < n_obs - 1)



# def parse_preds(s):
#     """Turn a ';'-joined string into a list of ints (or empty)."""
#     if pd.isna(s) or not s:
#         return []
#     return [int(x) for x in s.split(";") if x.strip()]


# def main():
    
#     df = pd.read_csv("results.csv", dtype=str).fillna("")

#     annotations = json.load(open("annotations.json", "r"))

#     rows = []
#     for _, row in df.iterrows():
#         ds = row["dataset"]

#         csv_path = os.path.join("datasets", "csv", f"{ds}.csv")
#         data = pd.read_csv(csv_path, header=None)
#         n_obs = len(data)

#         for alg in ["BOCPD", "CUSUM", "MicroWatch", "PELT", "BOCPDMS_univ", "BOCPDMS_multi"]:
#             raw = row.get(alg, "")
#             preds = parse_preds(raw)
#             preds = clean_cps(preds, n_obs)

#             f1, prec, rec = f_measure(annotations[ds], preds, return_PR=True)
#             cov = covering(annotations[ds], preds, n_obs)

#             rows.append({
#                 "dataset": ds,
#                 "algorithm": alg,
#                 "f1": f1,
#                 "precision": prec,
#                 "recall": rec,
#                 "cover": cov
#             })

#     perf = pd.DataFrame(rows)
#     perf.to_csv("accuracy.csv", index=False)
#     print("→ accuracy.csv written")


# if __name__ == "__main__":
#     main()


import json
import pandas as pd
import numpy as np
from pathlib import Path

# ——— port of the reference project’s metrics ———

def true_positives(T, X, margin=5):
    X = set(X)
    TP = set()
    for tau in T:
        close = [(abs(tau - x), x) for x in X if abs(tau - x) <= margin]
        if not close:
            continue
        dist, xstar = min(close)
        TP.add(tau)
        X.remove(xstar)
    return TP

def f_measure(annotations, predictions, margin=5, alpha=0.5, return_PR=False):
    # build per-horizon sets T1, T2, … then add zero
    Tks = {k+1: set(annotations[uid]) for k, uid in enumerate(annotations)}
    for Tk in Tks.values():
        Tk.add(0)
    # flatten into T*
    Tstar = set().union(*Tks.values())
    X = set(predictions)
    X.add(0)

    # precision on entire X
    P = len(true_positives(Tstar, X, margin)) / len(X)

    # recall averaged over horizons
    K = len(Tks)
    R = sum(len(true_positives(Tks[k], X, margin)) / len(Tks[k]) for k in Tks) / K

    # generalized F
    F = (P*R) / (alpha*R + (1-alpha)*P) if (P+R) > 0 else 0.0
    if return_PR:
        return F, P, R
    return F

def overlap(A, B):
    return len(A & B) / len(A | B)

def partition_from_cps(cps, n_obs):
    cps = sorted(set(cps))
    parts = []
    prev = 0
    for cp in cps:
        parts.append(set(range(prev, cp)))
        prev = cp
    parts.append(set(range(prev, n_obs)))
    return parts

def covering(annotations, predictions, n_obs):
    Ak = {k+1: partition_from_cps(annotations[uid], n_obs)
          for k, uid in enumerate(annotations)}
    pX = partition_from_cps(predictions, n_obs)

    def cover_single(S, Sp):
        T = sum(len(r) for r in Sp)
        C = 0.0
        for R in S:
            C += len(R) * max(overlap(R, Rprime) for Rprime in Sp)
        return C / T

    return sum(cover_single(Ak[k], pX) for k in Ak) / len(Ak)

# ——— end metrics ———

def parse_preds(s):
    """turn “;”-joined string into list of ints; skip if empty or NaN"""
    if pd.isna(s) or not str(s).strip():
        return []
    return [int(x) for x in str(s).split(";") if x.strip()]

def main():
    DATA_DIR = Path("datasets/csv")
    # load your results from the tuner
    df = pd.read_csv("results.csv")
    # load all annotations
    with open("annotations.json") as f:
        ann = json.load(f)

    out = []
    for _, row in df.iterrows():
        ds = row["dataset"]
        # ground-truth cps (flatten across horizons)
        truth = set()
        for lst in ann.get(ds, {}).values():
            truth |= set(lst)

        # reload the original CSV to get n_obs
        path = DATA_DIR / f"{ds}.csv"
        data = pd.read_csv(path, header=None)
        n_obs = data.shape[0]

        for alg in ["BOCPD","CUSUM","MicroWatch","PELT",
                    "BOCPDMS_univ","BOCPDMS_multi"]:
            preds = parse_preds(row.get(alg, ""))
            if not preds:
                continue
            f1, prec, rec = f_measure(ann[ds], preds, return_PR=True)
            cov = covering(ann[ds], preds, n_obs)
            out.append({
                "dataset": ds,
                "algorithm": alg,
                "f1": f1,
                "precision": prec,
                "recall": rec,
                "cover": cov
            })

    perf = pd.DataFrame(out)
    perf.to_csv("accuracy.csv", index=False)
    print("→ accuracy.csv written")

if __name__ == "__main__":
    main()