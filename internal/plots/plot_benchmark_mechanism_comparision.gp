#!/usr/bin/gnuplot -p

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
set tics in scale 1 font ",18"
set xtics offset 0,-0.5

set style line 102 lc rgb color_graph_grid lt 0 lw 1
set grid back ls 102

set tmargin 8
set lmargin 13
set bmargin 6

set xlabel "payload size [kb]" font ",18" offset 0,-1.5
set ylabel "latency [µs]" font ",18" offset -1.5,0
set title "Benchmark\n{/*0.7 Arch Linux on AMD Ryzen 7 7840S}" font ",24"
set logscale x 2
set logscale y 10
set xrange [0:4000]
set yrange [0:5000]
set key left
set term png transparent truecolor size output_width,output_height
set output 'benchmark_mechanism.png'

set style line 1 \
    linecolor rgb color_iceoryx2 \
    linetype 1 linewidth 4 \
    pointtype 7 pointsize 1
set style line 2 \
    linecolor rgb color_iceoryx \
    linetype 1 linewidth 4 \
    pointtype 7 pointsize 1
set style line 3 \
    linecolor rgb color_message_queue \
    linetype 1 linewidth 4 \
    pointtype 7 pointsize 1
set style line 4 \
    linecolor rgb color_unix_domain_socket \
    linetype 1 linewidth 4 \
    pointtype 7 pointsize 1

plot 'benchmark_mechanism_comparision.dat' index 0 with linespoints linestyle 1 title "iceoryx2", \
     ''                      index 1 with linespoints linestyle 2 title "iceoryx", \
     ''                      index 2 with linespoints linestyle 3 title "message queue", \
     ''                      index 3 with linespoints linestyle 4 title "unix domain socket" \

system("convert benchmark_mechanism.png -background '#FFFFFFAA' -flatten benchmark_mechanism.png")
