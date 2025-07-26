import os
import json
import pandas as pd

# ---------- CONFIG ----------
RESULTS_CSV_PATH = r"results\results.csv"
GROUND_TRUTH_JSON_PATH = r"scripts\annotations.json"
OUTPUT_PATH = r"results\accuracy.csv"

MARGIN = 5   
ALPHA = 0.5  


def true_positives(T, X, margin=MARGIN):
    """OG logic: count true positives with ±margin tolerance."""
    X = set(X)
    TP = set()
    for tau in T:
        close = [(abs(tau - x), x) for x in X if abs(tau - x) <= margin]
        if not close:
            continue
        _, xstar = min(close)
        TP.add(tau)
        X.remove(xstar)
    return TP

def f_measure(annotations, predictions, margin=MARGIN, alpha=ALPHA, return_PR=False):
    """OG F-measure calculation."""
    Tks = {k + 1: set(annotations[uid]) for k, uid in enumerate(annotations)}
    for Tk in Tks.values():
        Tk.add(0)

    X = set(predictions)
    X.add(0)

    Tstar = set().union(*Tks.values())
    P = len(true_positives(Tstar, X.copy(), margin)) / len(X) if X else 0.0

    TPk = {k: true_positives(Tks[k], X.copy(), margin) for k in Tks}
    R = sum(len(TPk[k]) / len(Tks[k]) for k in Tks) / len(Tks) if Tks else 0.0

    denom = alpha * R + (1 - alpha) * P
    F = (P * R / denom) if denom > 0 else 0.0

    if return_PR:
        return F, P, R
    return F

def overlap(A, B):
    return len(A & B) / len(A | B) if A or B else 0.0

def partition_from_cps(locations, n_obs):
    cps = sorted(set(locations))
    partition, current = [], set()
    it = iter(cps)
    cp = next(it, None)
    for i in range(n_obs):
        if i == cp:
            if current:
                partition.append(current)
            current = set()
            cp = next(it, None)
        current.add(i)
    partition.append(current)
    return partition

def cover_single(S, Sprime):
    T = sum(len(r) for r in Sprime)
    C = sum(len(R) * max(overlap(R, Rp) for Rp in Sprime) for R in S)
    return C / T if T > 0 else 0.0

def covering(annotations, predictions, n_obs):
    Ak = {k + 1: partition_from_cps(annotations[uid], n_obs)
          for k, uid in enumerate(annotations)}
    pX = partition_from_cps(predictions, n_obs)
    scores = [cover_single(Ak[k], pX) for k in Ak]
    return sum(scores) / len(scores) if scores else 0.0

def clean_cps(locations, n_obs):
    """OG cleaning: throw away cp < 1 or ≥ n_obs−1."""
    return sorted(cp for cp in locations if 1 <= cp < n_obs - 1)

def parse_preds(s):
    """OG parser: ';'-joined integers."""
    if pd.isna(s) or not s.strip():
        return []
    return [int(x) for x in s.split(";") if x.strip()]



def main():
    df = pd.read_csv(RESULTS_CSV_PATH, dtype=str).fillna("")
    annotations = json.load(open(GROUND_TRUTH_JSON_PATH, "r"))

    rows = []
    for _, row in df.iterrows():
        ds = row["dataset"]
        if ds not in annotations:
            continue

        
        csv_path = os.path.join("datasets", "csv", f"{ds}.csv")
        if not os.path.exists(csv_path):
            continue
        n_obs = len(pd.read_csv(csv_path, header=None))

        preds = parse_preds(row["change_points"])
        preds = clean_cps(preds, n_obs)

        F, P, R = f_measure(annotations[ds], preds, return_PR=True)
        cov = covering(annotations[ds], preds, n_obs)

        rows.append({
            "dataset": ds,
            "algorithm": row["method"],  
            "f1": F,
            "precision": P,
            "recall": R,
            "cover": cov
        })

    perf = pd.DataFrame(rows)
    perf.to_csv(OUTPUT_PATH, index=False)
    print(f"accuracy.csv written → {OUTPUT_PATH}")
    print(perf)

if __name__ == "__main__":
    main()
