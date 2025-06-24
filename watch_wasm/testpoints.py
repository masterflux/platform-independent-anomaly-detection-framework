import matplotlib.pyplot as plt
import numpy as np

# load
data = np.loadtxt("input.csv", delimiter=",")
cps  = [48, 81, 126, 168, 201, 234, 279, 315, 348, 381, 414, 474, 507, 552]

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
