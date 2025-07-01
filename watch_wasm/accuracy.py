import pandas as pd
from pathlib import Path

def parse_preds(s: str):
    """Turn 'a;b;c' → [a,b,c], or [] if empty/NaN."""
    if pd.isna(s) or not s.strip():
        return []
    return [int(x) for x in s.split(";") if x.strip()]

def main():
    DATA_DIR = Path("datasets/csv")
    df = pd.read_csv("results.csv", dtype=str).fillna("")

    rows = []
    for _, row in df.iterrows():
        ds = row["dataset"]
        path = DATA_DIR / f"{ds}.csv"
        data = pd.read_csv(path, header=None)
        n_rows, n_cols = data.shape

        # define which algos to report
        if n_cols == 1:
            algs = ["BOCPD","CUSUM","MicroWatch","PELT","BOCPDMS_univ"]
        else:
            algs = ["MicroWatch","PELT","BOCPDMS_multi"]

        for alg in algs:
            preds = parse_preds(row.get(alg, ""))
            detected = len(preds)
            # |detected - total| / total * 100%
            acc = abs(detected - n_rows) / n_rows * 100.0

            rows.append({
                "dataset": ds,
                "algorithm": alg,
                "detected_cps": detected,
                "total_points": n_rows,
                "accuracy_%": acc
            })

    out = pd.DataFrame(rows)
    out.to_csv("size_accuracy.csv", index=False)
    print("→ size_accuracy.csv written")

if __name__ == "__main__":
    main()
