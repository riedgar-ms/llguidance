import matplotlib.pyplot as plt
import numpy as np

# Data
categories = ["0%-2% & !sliced", "2%-85% & !sliced", "85%+ & !sliced", "85%+ & sliced"]
percent_masks = [44.6, 1.1, 0.5, 53.8]
percent_time = [20.7, 11.0, 13.0, 55.0]
time_per_mask_us = [28, 576, 1577, 61]

# Normalize time/mask to 100% = 2000us
time_per_mask_norm = [x / 2000 * 100 for x in time_per_mask_us]

x = np.arange(len(categories))
width = 0.25

fig, ax = plt.subplots(figsize=(8, 4))

ax.bar(x - width, percent_masks, width, label='% masks', color='tab:blue')
ax.bar(x, percent_time, width, label='% time', color='tab:orange')
ax.bar(x + width, time_per_mask_norm, width, label='time/mask (100%=2ms)', color='tab:green')

ax.set_title('Number of masks and computation time, for different mask-fill levels')
ax.set_xticks(x)
ax.set_xticklabels(categories)
ax.set_ylim(0, 90)
ax.set_yticks(np.arange(0, 100, 10))
ax.set_yticklabels([f'{i}%' for i in range(0, 100, 10)])
ax.yaxis.grid(True, linestyle='--', alpha=0.6)
ax.legend()

plt.tight_layout()

id = "mask_plot"
plt.savefig(f"{id}.svg", dpi=300)
plt.savefig(f"{id}.png", dpi=300)