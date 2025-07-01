import pandas as pd
import matplotlib.pyplot as plt

# 1) Load your series (single‐column CSV of length ≥500)
df = pd.read_csv("datasets/csv/brent_spot.csv")  
values = df.iloc[:,0].values

# 2) Define first‐300 slice
n = 300
x = list(range(n))
y = values[:n]

# 3) Your CP lists
new_cp = [48, 96, 136, 194, 230, 277, 381, 466]
old_cp = [220, 221, 222]
# (add the rest of your indices here…)

# Keep only those < n
new_cp = [i for i in new_cp if i < n]
old_cp = [i for i in old_cp if i < n]

# 4) Plot side by side
fig, axes = plt.subplots(1, 2, figsize=(14, 5), sharey=True)

for ax, cps, title in zip(axes, [new_cp, old_cp], ["New Results", "Old Results"]):
    # plot time-series
    ax.plot(x, y, label="Price")
    # overlay CPs
    ax.scatter(
        cps,
        [y[i] for i in cps],
        color="red",
        s=60,
        zorder=5,
        label=f"{len(cps)} CPs"
    )
    ax.set_title(title)
    ax.set_xlabel("Sample index")
    ax.legend(loc="upper left")

axes[0].set_ylabel("Value")
plt.suptitle(f"Zoom on first {n} samples — change-points as red dots", y=1.02)
plt.tight_layout()
plt.show()
