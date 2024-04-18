#!/usr/bin/gnuplot -p

color_graph_caption='#000000'
color_graph_grid='#d6d7d9'
color_graph_box='#528c19'
output_height=600 # this is the width before rotating
output_width=1400 # this is the height before rotating

set style line 101 lc rgb color_graph_caption lt 1 lw 1
set border 3 front ls 101
set tics center nomirror out scale 1 font ",18"

set style line 102 lc rgb color_graph_grid lt 0 lw 1
set grid back ls 102

set tmargin 3
set lmargin 10
set rmargin 6
set bmargin 14

set xtic rotate by 90 scale 0
set ytics rotate by 90
set ylabel "iceoryx2 Latency On Different Platforms" font ",24"
set y2label 'latency [ns] (less is better)' offset 2.5 font ",18"
set xlabel ' '

# set xlabel "payload size [kb]" font ",18" offset 0,-1.5
# set ylabel "latency [µs]" font ",18" offset -1.5,0
# set title "iceoryx2 Latency On Different Platforms" font ",24"

set xtics left offset 0,-12 font ",14"
unset key

set term png transparent truecolor size output_height,output_width
set output 'benchmark_architecture.png'

set boxwidth 0.6
set style fill solid noborder
set yrange[0:1000]

plot 'benchmark_architecture_os_comparision.dat' using 0:3:xtic(sprintf("%s\n    %s", stringcolumn(1), stringcolumn(2))) with boxes lc rgb color_graph_box

system("convert -rotate 90 benchmark_architecture.png benchmark_architecture.png")
system("convert benchmark_architecture.png -background '#FFFFFFAA' -flatten benchmark_architecture.png")
