#!/usr/bin/gnuplot -p

color_graph_caption='#000000'
color_graph_grid='#d6d7d9'
color_graph_box='#738f4d'
color_iceoryx='#2697de'
color_iceoryx2='#97de26'
color_message_queue='#282828'
color_unix_domain_socket='#282828'
output_height=300
output_width=1200

set style line 101 lc rgb color_graph_caption lt 1 lw 1
set border 3 front ls 101
set tics nomirror out scale 0.75

set style line 102 lc rgb color_graph_grid lt 0 lw 1
set grid back ls 102

set xlabel "payload size in kilobyte"
set ylabel "latency in microseconds"
set title "benchmark"
set logscale x 2
set logscale y 2
set xrange [0:4096]
set yrange [0:3100]
set key left
set term png transparent truecolor size output_width,output_height
set output 'benchmark_mechanism.png'

set style line 1 \
    linecolor rgb color_iceoryx2 \
    linetype 1 linewidth 4 \
    pointtype 0 pointsize 1
set style line 2 \
    linecolor rgb color_iceoryx \
    linetype 1 linewidth 2 \
    pointtype 0 pointsize 1
set style line 3 \
    linecolor rgb color_message_queue \
    linetype 0 linewidth 2 \
    pointtype 0 pointsize 1
set style line 4 \
    linecolor rgb color_unix_domain_socket \
    linetype 5 linewidth 2 \
    pointtype 0 pointsize 1

plot 'benchmark_mechanism_comparision.dat' index 0 with linespoints linestyle 1 title "iceoryx2", \
     ''                      index 1 with linespoints linestyle 2 title "iceoryx", \
     ''                      index 2 with linespoints linestyle 3 title "message queue", \
     ''                      index 3 with linespoints linestyle 4 title "unix domain socket" \

system("convert benchmark_mechanism.png -trim +repage benchmark_mechanism.png")
