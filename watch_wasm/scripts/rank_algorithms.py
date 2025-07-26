import pandas as pd

# -------- CONFIG --------
INPUT_CSV = r"results\accuracy.csv"  
OUTPUT_RANKED = r"results\ranked_algorithms.csv"

df = pd.read_csv(INPUT_CSV)

metrics = ["f1", "precision", "recall", "cover"]
df[metrics] = df[metrics].apply(pd.to_numeric, errors="coerce")

agg_df = (
    df.groupby("algorithm")[metrics]
    .mean()
    .reset_index()
    .sort_values(by="f1", ascending=False)
)

agg_df = agg_df.sort_values(by=["f1", "cover"], ascending=False)
agg_df["rank"] = range(1, len(agg_df) + 1)

agg_df.to_csv(OUTPUT_RANKED, index=False)
print(f"✅ Global algorithm ranking saved → {OUTPUT_RANKED}\n")
print(agg_df)