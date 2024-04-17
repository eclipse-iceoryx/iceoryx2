#!/usr/bin/gnuplot -p

color_graph_caption='#000000'
color_graph_grid='#d6d7d9'
color_graph_box='#738f4d'
output_height=700
output_width=1200

set style line 101 lc rgb color_graph_caption lt 1 lw 1
set border 3 front ls 101
set tics nomirror out scale 0.75

set style line 102 lc rgb color_graph_grid lt 0 lw 1
set grid back ls 102

set xtic rotate by 90 scale 0
set ytics rotate by 90
set ylabel "iceoryx2 Latency On Different Platforms"
set y2label 'latency in nanoseconds (less is better)' offset 2.5
set xlabel ' '
set size 0.6, 1
set xtics left offset 0,-7 font ", 8"
set bmargin 8
unset key

set term png transparent truecolor size output_height,output_width
set output 'benchmark_architecture.png'

set boxwidth 0.6
set style fill solid noborder
set yrange[0:1000]

plot 'benchmark_architecture_os_comparision.dat' using 0:3:xtic(sprintf("%s\n    %s", stringcolumn(1), stringcolumn(2))) with boxes lc rgb color_graph_box

system("convert -rotate 90 benchmark_architecture.png benchmark_architecture.png")
system("convert benchmark_architecture.png -trim +repage benchmark_architecture.png")
