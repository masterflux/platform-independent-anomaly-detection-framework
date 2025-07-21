# import pandas as pd
# from pathlib import Path

# def parse_preds(s: str):
#     """Turn 'a;b;c' → [a,b,c], or [] if empty/NaN."""
#     if pd.isna(s) or not s.strip():
#         return []
#     return [int(x) for x in s.split(";") if x.strip()]


# def main():
#     DATA_DIR = Path("datasets/csv")
#     # Read the results produced by the Rust program
#     df = pd.read_csv("results/results.csv", dtype=str).fillna("")

#     rows = []
#     for _, row in df.iterrows():
#         ds = row["dataset"]
#         method = row["method"]
#         cp_field = row.get("change_points", "")
#         preds = parse_preds(cp_field)
#         detected = len(preds)

#         # Load original series to get its length
#         path = DATA_DIR / f"{ds}.csv"
#         data = pd.read_csv(path, header=None)
#         n_rows, _ = data.shape

#         # Compute a simple "accuracy" metric: |detected - truth_count| / truth_count * 100%
#         # Here we assume the "truth_count" is the total number of rows (or change-points expected).
#         # Adjust if you have ground truth counts elsewhere.
#         acc = abs(detected - n_rows) / n_rows * 100.0

#         rows.append({
#             "dataset": ds,
#             "algorithm": method,
#             "detected_cps": detected,
#             "total_points": n_rows,
#             "accuracy_%": acc
#         })

#     out = pd.DataFrame(rows)
#     out.to_csv("results/size_accuracy.csv", index=False)
#     print("→ size_accuracy.csv written")


# if __name__ == "__main__":
#     main()

####################################################################################
# import pandas as pd
# from pathlib import Path

# def parse_preds(s: str):
#     """Turn 'a;b;c' → [a,b,c], or [] if empty/NaN."""
#     if pd.isna(s) or not s.strip():
#         return []
#     return [int(x) for x in s.split(";") if x.strip()]

# def main():
#     DATA_DIR = Path("datasets/csv")
#     df = pd.read_csv("results/results.csv", dtype=str).fillna("")

#     rows = []
#     for _, row in df.iterrows():
#         ds = row["dataset"]
#         method = row["method"]
#         cp_field = row.get("change_points", "")
#         preds = parse_preds(cp_field)
#         detected = len(preds)

#         # Total points = number of rows in dataset
#         path = DATA_DIR / f"{ds}.csv"
#         data = pd.read_csv(path, header=None)
#         n_rows, _ = data.shape

#         rows.append({
#             "algorithm": method,
#             "detected_points": detected,
#             "total_points": n_rows
#         })

#     # Convert to DataFrame
#     df_points = pd.DataFrame(rows)

#     # Group by algorithm, summing detected & total points
#     summary = df_points.groupby("algorithm", as_index=False).sum()

#     # Compute accuracy
#     summary["accuracy_%"] = (abs(summary["detected_points"] - summary["total_points"])/ summary["total_points"]) * 100

#     # Reorder columns
#     summary = summary[["algorithm", "accuracy_%", "total_points", "detected_points"]]

#     # Save to CSV
#     summary.to_csv("results/aggregated_accuracy.csv", index=False)
#     print("→ aggregated_accuracy.csv written")
#     print(summary)

# if __name__ == "__main__":
#     main()



# #################################
import json
import pandas as pd
from pathlib import Path

# ---------- CONFIG ----------
RESULTS_CSV_PATH = r"D:\University\Dissertation\Code\platform-independent-anomaly-detection-framework\watch_wasm\results\results.csv"
GROUND_TRUTH_JSON_PATH = r"D:\University\Dissertation\Code\platform-independent-anomaly-detection-framework\watch_wasm\scripts\annotations.json"
OUTPUT_PATH = r"D:\University\Dissertation\Code\platform-independent-anomaly-detection-framework\results\algorithm_accuracy_summary.csv"
TOLERANCE = 10  # ±10 points tolerance

# ---------- LOAD DATA ----------
results_df = pd.read_csv(RESULTS_CSV_PATH)
with open(GROUND_TRUTH_JSON_PATH, "r") as f:
    ground_truth = json.load(f)

# ---------- PARSE GROUND TRUTH (AGGREGATE ALL VERSIONS) ----------
true_cps = {}
for dataset, versions in ground_truth.items():
    cps = []
    for v in versions.values():
        cps.extend(v)
    true_cps[dataset] = sorted(set(cps))  # unique + sorted

# ---------- HELPER FUNCTIONS ----------
def parse_preds(s: str):
    """Turn 'a;b;c' → [a,b,c], or [] if empty/NaN."""
    if pd.isna(s) or not s.strip():
        return []
    return [int(x) for x in s.split(";") if x.strip()]

def match_with_tolerance(detected, truth, tol):
    """
    Count true positives allowing ±tol tolerance.
    Once a truth point is matched, it cannot be reused.
    """
    truth = sorted(truth)
    detected = sorted(detected)
    tp = 0
    used_truth = set()
    for d in detected:
        for t in truth:
            if t not in used_truth and abs(d - t) <= tol:
                tp += 1
                used_truth.add(t)
                break
    return tp

# ---------- CALCULATE METRICS PER ALGO ----------
algo_stats = {}

for _, row in results_df.iterrows():
    dataset = row["dataset"]
    algo = row["method"]
    detected = parse_preds(row["change_points"])
    truth = true_cps.get(dataset, [])

    if not truth:  # skip datasets with no ground truth
        continue

    tp = match_with_tolerance(detected, truth, TOLERANCE)
    fp = len(detected) - tp
    fn = len(truth) - tp

    if algo not in algo_stats:
        algo_stats[algo] = {"tp": 0, "fp": 0, "fn": 0, "true_cps": 0, "detected_cps": 0}

    algo_stats[algo]["tp"] += tp
    algo_stats[algo]["fp"] += fp
    algo_stats[algo]["fn"] += fn
    algo_stats[algo]["true_cps"] += len(truth)
    algo_stats[algo]["detected_cps"] += len(detected)

# ---------- CREATE FINAL SUMMARY ----------
summary_rows = []
for algo, stats in algo_stats.items():
    tp, fp, fn = stats["tp"], stats["fp"], stats["fn"]
    precision = tp / (tp + fp) * 100 if (tp + fp) > 0 else 0
    recall = tp / (tp + fn) * 100 if (tp + fn) > 0 else 0
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0

    summary_rows.append({
        "algorithm": algo,
        "precision_%": round(precision, 2),
        "recall_%": round(recall, 2),
        "f1_score_%": round(f1, 2),
        "true_cps": stats["true_cps"],
        "detected_cps": stats["detected_cps"],
        "true_positives": tp
    })

summary_df = pd.DataFrame(summary_rows).sort_values(by="f1_score_%", ascending=False)
summary_df.to_csv(OUTPUT_PATH, index=False)

print(f"✅ Accuracy summary written to: {OUTPUT_PATH}")
print(summary_df)
