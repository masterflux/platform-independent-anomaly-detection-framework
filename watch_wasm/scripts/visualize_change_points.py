<<<<<<< HEAD
# import sys
# import os
# import pandas as pd
# import matplotlib.pyplot as plt

# # 1) get dataset name (argv or prompt)
# if len(sys.argv) > 1:
#     dataset = sys.argv[1]
# else:
#     dataset = input("Dataset name (without .csv): ").strip()

# # 2) build the two file paths under watch_wasm/
# data_path    = os.path.join("watch_wasm", "datasets", "csv", f"{dataset}.csv")
# results_path = os.path.join("watch_wasm", "results", "results.csv")

# # 3) load the series
# if not os.path.exists(data_path):
#     print(f"❌ Data file not found: {data_path}")
#     sys.exit(1)
# df = pd.read_csv(data_path, sep=None, engine="python")
# total = len(df)
# print(f"Total points = {total}")

# # 4) load results.csv
# if not os.path.exists(results_path):
#     print(f"❌ Results file not found: {results_path}")
#     sys.exit(1)
# res = pd.read_csv(results_path)
# sub = res[res["dataset"] == dataset]
# if sub.empty:
#     print(f"❌ No rows for '{dataset}' in results.csv")
#     sys.exit(1)

# # 5) parse change-point lists
# methods = sub["method"].tolist()
# cps = []
# for pts in sub["change_points"].fillna(""):
#     cps.append([int(x) for x in pts.split(";") if x.isdigit()])

# # 6) scatter-plot
# plt.figure(figsize=(8, 2 + len(methods)*0.4))
# for i, pts in enumerate(cps):
#     plt.scatter(pts, [i]*len(pts), s=30, label=methods[i])
# plt.axvline(total, linestyle="--", label=f"Length = {total}")

# plt.yticks(range(len(methods)), methods)
# plt.xlabel("Index in series")
# plt.title(f"Change-points for '{dataset}'")
# plt.legend(bbox_to_anchor=(1.05,1), loc="upper left")
# plt.tight_layout()
# plt.show()


import os
import csv
import pandas as pd
import matplotlib.pyplot as plt

def load_series(path):
    with open(path, newline='') as f:
        sample = f.read(4096)
        try:
            dialect = csv.Sniffer().sniff(sample)
            sep = dialect.delimiter
        except csv.Error:
            sep = ','

    # Read, skipping any badly‐formed lines
    return pd.read_csv(
        path,
        sep=sep,
        engine='python',
        on_bad_lines='skip'   
    )

#  Prompt for dataset
dataset = input("Enter dataset name (without .csv): ").strip()

# Paths
data_csv    = f"watch_wasm/datasets/csv/{dataset}.csv"
results_csv = "watch_wasm/results/results.csv"

#  Load the series robustly
if not os.path.exists(data_csv):
    print("Data file not found:", data_csv); exit(1)
df = load_series(data_csv)

# pick the value column
series = df.iloc[:,1] if df.shape[1] > 1 else df.iloc[:,0]
y = series.values
x = list(range(len(y)))
print(f"Loaded {len(y)} samples from {os.path.basename(data_csv)}")

#  Load results.csv
if not os.path.exists(results_csv):
    print("Results file not found:", results_csv); exit(1)
res = pd.read_csv(results_csv)
sub = res[res["dataset"] == dataset]
if sub.empty:
    print(f"No entries for '{dataset}' in {results_csv}"); exit(1)

#  Parse per-algorithm CP lists
methods  = sub["method"].tolist()
cp_lists = [
    [int(tok) for tok in pts.split(";") if tok.isdigit()]
    for pts in sub["change_points"].fillna("")
]

# filter out‐of‐bounds
for i, cps in enumerate(cp_lists):
    valid = [c for c in cps if 0 <= c < len(y)]
    if len(valid) != len(cps):
        print(f"Dropped {len(cps)-len(valid)} invalid CPs from {methods[i]}")
    cp_lists[i] = valid

#  Plot full series + coloured dots per algorithm
plt.figure(figsize=(12,4))
plt.plot(x, y, color="steelblue", label="Value")

cmap = plt.get_cmap("tab10", len(methods))
for idx, (m, cps) in enumerate(zip(methods, cp_lists)):
    if not cps: 
        continue
    plt.scatter(
        cps, 
        [y[i] for i in cps],
        s=60,
        color=cmap(idx),
        label=m,
        edgecolor="k"
    )

plt.xlabel("Sample index")
plt.ylabel("Value")
plt.title(f"Detected Change-Points by Algorithm for '{dataset}'")
plt.legend(bbox_to_anchor=(1.02,1), loc="upper left")
plt.tight_layout()
plt.show()
=======
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
>>>>>>> d719506060314435b622b74a3c80976d99a80752
