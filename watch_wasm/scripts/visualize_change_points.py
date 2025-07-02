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