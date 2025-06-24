import matplotlib.pyplot as plt
import numpy as np

# load
data = np.loadtxt("datasets/csv/bank.csv", delimiter=",")
cps  = [18, 41, 49, 55, 70, 79, 90, 104, 126, 139, 153, 169, 177, 185, 197, 205, 214, 231, 243, 256, 264, 275, 286, 294, 306, 317, 328, 337, 351, 375, 386, 405, 414, 421, 432, 443, 452, 476, 485, 494, 507, 518, 528, 534, 542, 549, 562, 570]

# only keep CPs before x=300
cps_small = [cp for cp in cps if cp < 600]

plt.figure(figsize=(10,4))
plt.plot(data, lw=1, color='steelblue', label='Value')

# highlight only the early CPs:
plt.scatter(cps_small, data[cps_small],
            c='crimson', s=80, zorder=5,
            label=f'{len(cps_small)} CPs (x<300)')

# zoom in
plt.xlim(0, 600)
plt.ylim(min(data[:600])*0.9, max(data[:600])*1.1)

plt.title("Zoom on first 300 samples — change-points as red dots")
plt.xlabel("Sample index")
plt.ylabel("Value")
plt.legend(loc='upper right')
plt.tight_layout()
plt.show()
