import pandas as pd
import matplotlib.pyplot as plt


df = pd.read_csv("datasets/csv/brent_spot.csv")  
values = df.iloc[:,0].values


n = 300
x = list(range(n))
y = values[:n]


new_cp = [48, 96, 136, 194, 230, 277, 381, 466]
old_cp = [220, 221, 222]


new_cp = [i for i in new_cp if i < n]
old_cp = [i for i in old_cp if i < n]


fig, axes = plt.subplots(1, 2, figsize=(14, 5), sharey=True)

for ax, cps, title in zip(axes, [new_cp, old_cp], ["New Results", "Old Results"]):
    
    ax.plot(x, y, label="Price")
    
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
