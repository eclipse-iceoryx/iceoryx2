#!/usr/bin/gnuplot -p

data_file="benchmark_mechanism_comparision_i7_13700h.dat"
system_type="Arch Linux on Intel i7 13700h"

color_graph_caption='#595959'
color_graph_grid='#d6d7d9'
color_graph_box='#738f4d'
color_iceoryx='#228bc0'
color_iceoryx2='#528c19'
color_message_queue='#c0a622'
color_unix_domain_socket='#c06622'
output_height=600
output_width=1400

set style line 101 lc rgb color_graph_caption lt 1 lw 2
set border 15 back ls 101
set tics in scale 0.75 font ",20"
set xtics offset 0,-0.5
set ytics mirror

set style line 102 lc rgb color_graph_grid lt 0 lw 1
set grid back ls 102

set tmargin 7
set lmargin 13
set rmargin 5
set bmargin 6

set title "Benchmark" font ",36" offset 0,-0.75
set x2label system_type font ",24"
set xlabel "payload size [KB]" font ",24" offset 0,-1.5
set ylabel "latency [µs]" font ",24" offset -1.5,0
set logscale x 2
set logscale y 10
set xrange [0:4096]
set yrange [0.08:2000]
set key left enhanced font ",16" width -3
set object rectangle from screen -0.1,-0.1 to screen 1.1,1.1 fs noborder solid 0.7 fc rgb "#FFFFFF" behind
set term svg enhanced font "sans" size output_width,output_height
set output 'benchmark_mechanism.svg'

set style line 1 \
    linecolor rgb color_iceoryx2 \
    linetype 1 linewidth 4 \
    pointtype 7 pointsize 1.2
set style line 2 \
    linecolor rgb color_iceoryx \
    linetype 1 linewidth 4 \
    pointtype 13 pointsize 1.2
set style line 3 \
    linecolor rgb color_message_queue \
    linetype 1 linewidth 4 \
    pointtype 5 pointsize 1.2
set style line 4 \
    linecolor rgb color_unix_domain_socket \
    linetype 1 linewidth 4 \
    pointtype 9 pointsize 1.2

plot data_file index 0 with linespoints linestyle 1 title "iceoryx2", \
     ''                      index 1 with linespoints linestyle 2 title "iceoryx", \
     ''                      index 2 with linespoints linestyle 3 title "message queue", \
     ''                      index 3 with linespoints linestyle 4 title "unix domain socket" \
