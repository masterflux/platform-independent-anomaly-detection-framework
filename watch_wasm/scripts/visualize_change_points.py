# #!/usr/bin/env python3
# import os
# import pandas as pd
# import numpy as np
# import matplotlib.pyplot as plt

# # 1) read the detected‐points table
# df = pd.read_csv("results/results.csv", dtype=str).fillna("")

# # 2) utility to parse "12;34;56" → [12,34,56]
# def parse_pts(s):
#     return [int(x) for x in s.split(';') if x.strip().isdigit()]

# # 3) make output dir
# os.makedirs("plots", exist_ok=True)

# for _, row in df.iterrows():
#     name = row["dataset"]
#     # load the data
#     data = pd.read_csv(f"datasets/csv/{name}.csv", header=None)
#     # collapse to a single series if multivariate
#     if data.shape[1] > 1:
#         ts = data.mean(axis=1).values
#     else:
#         ts = data.iloc[:, 0].values

#     n = len(ts)
#     x = np.arange(n)

#     plt.figure(figsize=(10,4))
#     plt.plot(x, ts, label="series", lw=1.2, color="black")

#     methods = ["BOCPD","CUSUM","MicroWatch","PELT","BOCPDMS_univ","BOCPDMS_multi"]
#     colors = {
#         "BOCPD":"C0","CUSUM":"C1","MicroWatch":"C2",
#         "PELT":"C3","BOCPDMS_univ":"C4","BOCPDMS_multi":"C5"
#     }

#     for m in methods:
#         raw = row.get(m, "")
#         pts = [p for p in parse_pts(raw) if 0 <= p < n]
#         # now safe to plot
#         for p in pts:
#             plt.axvline(p, color=colors[m], alpha=0.6, linestyle="--")
#         if pts:
#             plt.scatter(pts, ts[pts], color=colors[m], s=25, label=f"{m} ({len(pts)})")

#     plt.title(f"{name}  (n={n})")
#     plt.xlabel("t")
#     plt.ylabel("value")
#     plt.legend(ncol=2, fontsize="small", loc="upper right")
#     plt.tight_layout()
#     plt.savefig(f"plots/{name}.png", dpi=150)
#     plt.close()


#!/usr/bin/env python3
import argparse
import os
import pandas as pd
import matplotlib.pyplot as plt

def parse_cp_list(s):
    """Turn '12;34;56' or '' into [12,34,56] or []."""
    if pd.isna(s) or str(s).strip()=="":
        return []
    return [int(x) for x in str(s).split(";")]

def load_series(path):
    """Load a CSV (no header) and return a 1D numpy array.
    If multivariate, collapse by row-mean."""
    df = pd.read_csv(path, header=None)
    if df.shape[1] > 1:
        return df.mean(axis=1).values
    else:
        return df.iloc[:,0].values

def plot_stacked(dataset, results_csv="results/results.csv", data_dir="datasets/csv"):
    # --- load change‐point table ---
    df = pd.read_csv(results_csv)
    row = df[df["dataset"] == dataset]
    if row.empty:
        raise ValueError(f"{dataset!r} not found in {results_csv}")
    row = row.iloc[0]

    # parse all methods
    methods = [c for c in df.columns if c!="dataset"]
    cps = {m: parse_cp_list(row[m]) for m in methods}

    # --- load time-series ---
    csv_path = os.path.join(data_dir, f"{dataset}.csv")
    ts = load_series(csv_path)
    T = len(ts)

    # --- build figure ---
    n = len(methods)+1
    fig, axes = plt.subplots(n, 1, sharex=True, figsize=(12, 2*n))
    # top: raw series
    axes[0].plot(ts, linewidth=1, color="k")
    axes[0].set_ylabel("value")
    axes[0].set_title(f"{dataset}  (T={T})")

    # each detector
    for i, m in enumerate(methods, start=1):
        ax = axes[i]
        pts = cps[m]
        # draw vertical markers:
        ax.scatter(pts, ts[pts], marker="|", s=200, color=f"C{i%10}")
        ax.set_ylabel(m, rotation=0, ha="right")
        ax.set_yticks([])
    axes[-1].set_xlabel("time index")
    plt.tight_layout()
    plt.show()

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--dataset", required=True,
                   help="dataset name (without .csv)")
    p.add_argument("--results", default="results/results.csv",
                   help="path to results.csv")
    p.add_argument("--data-dir", default="datasets/csv",
                   help="where the raw CSVs live")
    args = p.parse_args()
    plot_stacked(args.dataset, args.results, args.data_dir)
