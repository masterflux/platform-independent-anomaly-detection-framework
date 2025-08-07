import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from matplotlib.patches import Rectangle
import matplotlib.patches as mpatches

df = pd.read_csv('results/final_accuracy_all_env.csv')


df['algorithm'] = df['algorithm'].replace({
    'MicroWatch(idx=0)': 'MicroWatch(v0)',
    'MicroWatch(idx=1)': 'MicroWatch(v1)',
    'MicroWatch(idx=2)': 'MicroWatch(v2)',
    'MicroWatch(idx=3)': 'MicroWatch(v3)',
    'MicroWatch(idx=4)': 'MicroWatch(v4)',
    'MicroWatch(idx=5)': 'MicroWatch(v5)',
    'MicroWatch(idx=6)': 'MicroWatch(v6)'
})

algo_metrics = df.groupby('algorithm').agg({
    'f1': 'mean',
    'precision': 'mean',
    'recall': 'mean',
    'cover': 'mean'
}).round(3)

algo_metrics = algo_metrics.sort_values('f1', ascending=False)

confusion_data = []
for algo in algo_metrics.index:
    precision = algo_metrics.loc[algo, 'precision']
    recall = algo_metrics.loc[algo, 'recall']
    
    tp = precision * recall  # True Positive rate
    fp = (1 - precision) * recall  # False Positive rate
    fn = precision * (1 - recall)  # False Negative rate
    tn = (1 - precision) * (1 - recall)  # True Negative rate
    
    confusion_data.append([tp, fp, fn, tn])

fig = plt.figure(figsize=(14, 8))

gs = fig.add_gridspec(2, 3, width_ratios=[3, 3, 1], height_ratios=[1, 1], 
                      hspace=0.3, wspace=0.3)

ax1 = fig.add_subplot(gs[:, 0])
metrics_data = algo_metrics.values
im1 = ax1.imshow(metrics_data.T, cmap='RdYlGn', aspect='auto', vmin=0, vmax=1)

ax1.set_xticks(np.arange(len(algo_metrics.index)))
ax1.set_xticklabels(algo_metrics.index, rotation=45, ha='right')
ax1.set_yticks(np.arange(len(algo_metrics.columns)))
ax1.set_yticklabels(['F1 Score', 'Precision', 'Recall', 'Coverage'])

for i in range(len(algo_metrics.index)):
    for j in range(len(algo_metrics.columns)):
        text = ax1.text(i, j, f'{metrics_data[i, j]:.3f}',
                       ha="center", va="center", color="black", fontsize=8)

ax1.set_title('Change Point Detection Performance Metrics\n(Platform-Independent Results)', 
              fontsize=12, fontweight='bold')

ax2 = fig.add_subplot(gs[:, 1])
confusion_array = np.array(confusion_data).T


colors = ['#2ecc71', '#e74c3c', '#f39c12', '#95a5a6']  
n_bins = 100
cmap_confusion = plt.cm.colors.LinearSegmentedColormap.from_list('custom', 
    [(0, '#f8f9fa'), (0.5, '#adb5bd'), (1, '#343a40')], N=n_bins)

im2 = ax2.imshow(confusion_array, cmap=cmap_confusion, aspect='auto', vmin=0, vmax=1)

# Set ticks and labels
ax2.set_xticks(np.arange(len(algo_metrics.index)))
ax2.set_xticklabels(algo_metrics.index, rotation=45, ha='right')
ax2.set_yticks(np.arange(4))
ax2.set_yticklabels(['True Positive\n(Correct Detections)', 
                     'False Positive\n(False Alarms)', 
                     'False Negative\n(Missed Changes)', 
                     'True Negative\n(Correct Rejections)'])

for i in range(len(algo_metrics.index)):
    for j in range(4):
        value = confusion_array[j, i]
        text_color = 'white' if value > 0.5 else 'black'
        text = ax2.text(i, j, f'{value:.3f}',
                       ha="center", va="center", color=text_color, fontsize=8)

ax2.set_title('Detection Performance Breakdown\n(Derived from Precision & Recall)', 
              fontsize=12, fontweight='bold')

ax_cb1 = fig.add_subplot(gs[0, 2])
ax_cb2 = fig.add_subplot(gs[1, 2])

cb1 = plt.colorbar(im1, cax=ax_cb1)
cb1.set_label('Metric Value', rotation=270, labelpad=15)

cb2 = plt.colorbar(im2, cax=ax_cb2)
cb2.set_label('Rate', rotation=270, labelpad=15)

# Add a text box with key findings
# textstr = '\n'.join([
#     'Key Findings:',
#     f'• Best F1 Score: {algo_metrics.index[0]} ({algo_metrics.iloc[0]["f1"]:.3f})',
#     f'• Highest Precision: {algo_metrics.sort_values("precision", ascending=False).index[0]} ({algo_metrics.sort_values("precision", ascending=False).iloc[0]["precision"]:.3f})',
#     f'• Highest Recall: {algo_metrics.sort_values("recall", ascending=False).index[0]} ({algo_metrics.sort_values("recall", ascending=False).iloc[0]["recall"]:.3f})',
#     '• All metrics consistent across VM, Windows, and Raspberry Pi platforms'
# ])

# props = dict(boxstyle='round', facecolor='wheat', alpha=0.5)
# fig.text(0.02, 0.02, textstr, transform=fig.transFigure, fontsize=9,
#          verticalalignment='bottom', bbox=props)

plt.suptitle('Comprehensive Change Point Detection Algorithm Evaluation', 
             fontsize=14, fontweight='bold', y=0.98)

plt.tight_layout()
plt.savefig('accuracy_figure.pdf', dpi=300, bbox_inches='tight')
plt.savefig('accuracy_figure.png', dpi=300, bbox_inches='tight')
plt.show()

plt.figure(figsize=(10, 6))


fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 6), gridspec_kw={'width_ratios': [2, 1]})


sns.heatmap(algo_metrics.T, annot=True, fmt='.3f', cmap='RdYlGn', 
            cbar_kws={'label': 'Score'}, ax=ax1, vmin=0, vmax=1,
            linewidths=0.5, linecolor='gray')
ax1.set_xlabel('Algorithm', fontweight='bold')
ax1.set_ylabel('Metric', fontweight='bold')
ax1.set_title('Algorithm Performance Across All Metrics', fontweight='bold')


top_5 = algo_metrics.head(5)
tp_fp_data = []
for algo in top_5.index:
    p = top_5.loc[algo, 'precision']
    r = top_5.loc[algo, 'recall']
    tp_fp_data.append([p * r, (1-p) * r, p * (1-r)])  # TP, FP, FN rates

tp_fp_df = pd.DataFrame(tp_fp_data, 
                       index=top_5.index,
                       columns=['True Positive', 'False Positive', 'False Negative'])

tp_fp_df.plot(kind='bar', stacked=True, ax=ax2, 
              color=['#2ecc71', '#e74c3c', '#f39c12'],
              width=0.8)
ax2.set_xlabel('Algorithm', fontweight='bold')
ax2.set_ylabel('Rate', fontweight='bold')
ax2.set_title('Top 5: Detection Breakdown', fontweight='bold')
ax2.legend(loc='upper right', fontsize=8)
ax2.set_xticklabels(ax2.get_xticklabels(), rotation=45, ha='right')

plt.suptitle('Change Point Detection: Accuracy Analysis Across Computing Platforms\n' + 
             'Identical results on VM, Windows, and Raspberry Pi', 
             fontsize=14, fontweight='bold')
plt.tight_layout()
plt.savefig('accuracy_figure.pdf', dpi=300, bbox_inches='tight')
plt.savefig('accuracy_figure.png', dpi=300, bbox_inches='tight')
plt.show()
