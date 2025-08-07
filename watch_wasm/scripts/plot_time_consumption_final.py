import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from matplotlib.patches import Rectangle

def create_time_consumption_graphs():
    """
    Creates multiple visualization options for time consumption data
    """
    
    df = pd.read_csv('comprehensive_time_table.csv')
    
    df['Algorithm'] = df['Algorithm'].replace({
    'MicroWatch(idx=0)': 'MicroWatch(v0)',
    'MicroWatch(idx=1)': 'MicroWatch(v1)',
    'MicroWatch(idx=2)': 'MicroWatch(v2)',
    'MicroWatch(idx=3)': 'MicroWatch(v3)',
    'MicroWatch(idx=4)': 'MicroWatch(v4)',
    'MicroWatch(idx=5)': 'MicroWatch(v5)',
    'MicroWatch(idx=6)': 'MicroWatch(v6)'
})

    time_cols = ['Windows (ms)', 'Ubuntu (ms)', 'Raspberry Pi (ms)']
    for col in time_cols:
        df[col] = pd.to_numeric(df[col], errors='coerce')
    
    datasets = df['Dataset'].unique()
    algorithms = df['Algorithm'].unique()
    
    print(f"Found {len(datasets)} datasets and {len(algorithms)} algorithms")
    
    import os
    os.makedirs('graphs', exist_ok=True)
    

    print("\n1. Creating heatmap...")
    create_heatmap(df, datasets, algorithms)
    
    print("\n2. Creating grouped bar charts...")
    create_grouped_bar_charts(df, datasets)
    
    print("\n3. Creating platform comparison chart...")
    create_platform_comparison(df)
    
    print("\n4. Creating algorithm performance overview...")
    create_algorithm_overview(df)
    
    print("\n5. Creating log scale comparison...")
    create_log_scale_comparison(df)
    
    print("\nAll graphs saved to 'graphs' directory!")
    print("\nGraph files created:")
    print("  1. heatmap_all_data.png - Overview of all data")
    print("  2. grouped_bars_[dataset].png - Detailed view for each dataset")
    print("  3. platform_comparison.png - Speedup analysis")
    print("  4. algorithm_overview.png - Average performance by algorithm")
    print("  5. log_scale_comparison.png - Log scale view (good for PELT)")

def create_heatmap(df, datasets, algorithms):
    """Create a heatmap showing all time data"""
    
    fig, axes = plt.subplots(1, 3, figsize=(20, 10))
    platforms = ['Windows (ms)', 'Ubuntu (ms)', 'Raspberry Pi (ms)']
    
    for idx, platform in enumerate(platforms):
        pivot_data = df.pivot(index='Algorithm', columns='Dataset', values=platform)
        
        ax = axes[idx]
        sns.heatmap(pivot_data, 
                    annot=True, 
                    fmt='.1f', 
                    cmap='YlOrRd',
                    ax=ax,
                    cbar_kws={'label': 'Time (ms)'},
                    linewidths=0.5)
        
        ax.set_title(f'{platform.replace(" (ms)", "")} Execution Time', fontsize=14, fontweight='bold')
        ax.set_xlabel('Dataset', fontsize=12)
        ax.set_ylabel('Algorithm', fontsize=12)
        
        ax.set_xticklabels(ax.get_xticklabels(), rotation=45, ha='right')
    
    plt.suptitle('Execution Time Heatmap - All Platforms', fontsize=16, fontweight='bold')
    plt.tight_layout()
    plt.savefig('graphs/heatmap_all_data.png', dpi=300, bbox_inches='tight')
    plt.close()

def create_grouped_bar_charts(df, datasets):
    """Create grouped bar charts for each dataset"""
    
    datasets_per_figure = 4
    num_figures = (len(datasets) + datasets_per_figure - 1) // datasets_per_figure
    
    for fig_idx in range(num_figures):
        start_idx = fig_idx * datasets_per_figure
        end_idx = min(start_idx + datasets_per_figure, len(datasets))
        current_datasets = datasets[start_idx:end_idx]
        
        fig, axes = plt.subplots(2, 2, figsize=(15, 12))
        axes = axes.flatten()
        
        for idx, dataset in enumerate(current_datasets):
            ax = axes[idx]
            
            dataset_df = df[df['Dataset'] == dataset].copy()
            
            x = np.arange(len(dataset_df))
            width = 0.25
            
            bars1 = ax.bar(x - width, dataset_df['Windows (ms)'], width, label='Windows', color='#1f77b4')
            bars2 = ax.bar(x, dataset_df['Ubuntu (ms)'], width, label='Ubuntu', color='#ff7f0e')
            bars3 = ax.bar(x + width, dataset_df['Raspberry Pi (ms)'], width, label='Raspberry Pi', color='#2ca02c')
            
            ax.set_xlabel('Algorithm', fontsize=10)
            ax.set_ylabel('Time (ms)', fontsize=10)
            ax.set_title(f'Dataset: {dataset}', fontsize=12, fontweight='bold')
            ax.set_xticks(x)
            ax.set_xticklabels(dataset_df['Algorithm'], rotation=45, ha='right')
            ax.legend()
            ax.grid(True, alpha=0.3, axis='y')
            
            def autolabel(bars):
                for bar in bars:
                    height = bar.get_height()
                    if height > 0.01:  
                        ax.annotate(f'{height:.1f}',
                                   xy=(bar.get_x() + bar.get_width() / 2, height),
                                   xytext=(0, 3),
                                   textcoords="offset points",
                                   ha='center', va='bottom',
                                   fontsize=8)
            
            if len(dataset_df) <= 10:
                autolabel(bars1)
                autolabel(bars2)
                autolabel(bars3)
        
        for idx in range(len(current_datasets), len(axes)):
            axes[idx].set_visible(False)
        
        plt.suptitle(f'Execution Time Comparison - Datasets {start_idx+1} to {end_idx}', 
                     fontsize=14, fontweight='bold')
        plt.tight_layout()
        plt.savefig(f'graphs/grouped_bars_set{fig_idx+1}.png', dpi=300, bbox_inches='tight')
        plt.close()

def create_platform_comparison(df):
    """Create platform comparison showing relative performance"""
    
    df['Ubuntu_Speedup'] = df['Ubuntu (ms)'] / df['Windows (ms)']
    df['RaspPi_Speedup'] = df['Raspberry Pi (ms)'] / df['Windows (ms)']
    
    speedup_df = df.groupby('Algorithm')[['Ubuntu_Speedup', 'RaspPi_Speedup']].mean().reset_index()
    
    fig, ax = plt.subplots(figsize=(12, 8))
    
    x = np.arange(len(speedup_df))
    width = 0.35
    
    bars1 = ax.bar(x - width/2, speedup_df['Ubuntu_Speedup'], width, 
                    label='Ubuntu vs Windows', color='#ff7f0e')
    bars2 = ax.bar(x + width/2, speedup_df['RaspPi_Speedup'], width, 
                    label='Raspberry Pi vs Windows', color='#2ca02c')
    
    ax.axhline(y=1, color='gray', linestyle='--', alpha=0.7, label='Same as Windows')
    
    ax.set_xlabel('Algorithm', fontsize=12)
    ax.set_ylabel('Relative Execution Time (vs Windows)', fontsize=12)
    ax.set_title('Platform Performance Comparison (Relative to Windows)', fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(speedup_df['Algorithm'], rotation=45, ha='right')
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y')
    
    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            ax.annotate(f'{height:.2f}x',
                       xy=(bar.get_x() + bar.get_width() / 2, height),
                       xytext=(0, 3),
                       textcoords="offset points",
                       ha='center', va='bottom')
    
    plt.tight_layout()
    plt.savefig('graphs/platform_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()

def create_algorithm_overview(df):
    """Create overview of algorithm performance across all datasets"""
    
    algo_stats = df.groupby('Algorithm')[['Windows (ms)', 'Ubuntu (ms)', 'Raspberry Pi (ms)']].agg(['mean', 'std'])
    
    fig, ax = plt.subplots(figsize=(12, 8))
    
    algorithms = algo_stats.index
    x = np.arange(len(algorithms))
    width = 0.25
    
    windows_means = algo_stats[('Windows (ms)', 'mean')]
    windows_stds = algo_stats[('Windows (ms)', 'std')]
    ubuntu_means = algo_stats[('Ubuntu (ms)', 'mean')]
    ubuntu_stds = algo_stats[('Ubuntu (ms)', 'std')]
    rasppi_means = algo_stats[('Raspberry Pi (ms)', 'mean')]
    rasppi_stds = algo_stats[('Raspberry Pi (ms)', 'std')]
    
    ax.bar(x - width, windows_means, width, yerr=windows_stds, 
           label='Windows', color='#1f77b4', capsize=5)
    ax.bar(x, ubuntu_means, width, yerr=ubuntu_stds, 
           label='Ubuntu', color='#ff7f0e', capsize=5)
    ax.bar(x + width, rasppi_means, width, yerr=rasppi_stds, 
           label='Raspberry Pi', color='#2ca02c', capsize=5)
    
    ax.set_xlabel('Algorithm', fontsize=12)
    ax.set_ylabel('Average Time (ms)', fontsize=12)
    ax.set_title('Average Algorithm Performance Across All Datasets', fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(algorithms, rotation=45, ha='right')
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y')
    
    plt.tight_layout()
    plt.savefig('graphs/algorithm_overview.png', dpi=300, bbox_inches='tight')
    plt.close()

def create_log_scale_comparison(df):
    """Create log scale comparison - useful when PELT is much slower"""
    
    mean_times = df.groupby('Algorithm')[['Windows (ms)', 'Ubuntu (ms)', 'Raspberry Pi (ms)']].mean()
    
    fig, ax = plt.subplots(figsize=(12, 8))
    
    platforms = ['Windows (ms)', 'Ubuntu (ms)', 'Raspberry Pi (ms)']
    colors = ['#1f77b4', '#ff7f0e', '#2ca02c']
    markers = ['o', 's', '^']
    
    for platform, color, marker in zip(platforms, colors, markers):
        ax.scatter(range(len(mean_times)), mean_times[platform], 
                  label=platform.replace(' (ms)', ''), 
                  color=color, marker=marker, s=100)
        ax.plot(range(len(mean_times)), mean_times[platform], 
                color=color, alpha=0.5, linestyle='--')
    
    ax.set_yscale('log')
    ax.set_xlabel('Algorithm', fontsize=12)
    ax.set_ylabel('Average Time (ms) - Log Scale', fontsize=12)
    ax.set_title('Algorithm Performance Comparison (Log Scale)', fontsize=14, fontweight='bold')
    ax.set_xticks(range(len(mean_times)))
    ax.set_xticklabels(mean_times.index, rotation=45, ha='right')
    ax.legend()
    ax.grid(True, alpha=0.3, which='both')
    
    plt.tight_layout()
    plt.savefig('graphs/log_scale_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()

if __name__ == "__main__":
    create_time_consumption_graphs()