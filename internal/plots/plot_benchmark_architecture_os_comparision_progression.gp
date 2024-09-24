#!/usr/bin/gnuplot -p

color_graph_caption='#595959'
color_graph_grid='#d6d7d9'
color_graph_box_old='#bb000000'
color_graph_box_new='#44528c19'
output_height=650
output_width=1400

set style line 101 lc rgb color_graph_caption lt 1 lw 1
set tics center nomirror out scale 1 font ",20"

set style line 102 lc rgb color_graph_grid lt 0 lw 1
set grid back ls 102

set tmargin 5
set lmargin 25
set rmargin 5
set bmargin 5

set xtics in offset 0,-0.5 scale 0.5
set ytics right scale 0
set x2label "iceoryx2 Latency Improvement v0.3 -> v0.4" offset 0,-0.5 font ",36"
set xlabel 'latency [ns] (less is better)' offset 0,-1 font ",24"
set ylabel offset -10,0
set y2label "v0.3 [gray]  \n\nv0.4 [green]" offset -6,12 rotate by 0

set yrange [0:*]      # start at zero, find max from the data
set xrange [0:950]   # start at zero, find max from the data
set style fill solid  # solid color boxes

unset key

set object rectangle from screen -0.1,-0.1 to screen 1.1,1.1 fs noborder solid 0.7 fc rgb "#FFFFFF" behind
set term svg enhanced font "sans" size output_width,output_height
set output 'benchmark_progression.svg'

myBoxWidth = 0.7
set offsets 0,0,0.5-myBoxWidth/2.,0.5

plot \
     'archive/benchmark_architecture_os_comparision-v0.3.dat' using (0.5*$3):0:(0.5*$3):((myBoxWidth)/2.):ytic(sprintf("%s\n{/*0.8 %s}", stringcolumn(1), stringcolumn(2))) with boxxy lc rgb color_graph_box_old, \
     'benchmark_architecture_os_comparision.dat' using (0.5*$3):0:(0.5*$3):((myBoxWidth)/2.):ytic(sprintf("%s\n{/*0.8 %s}", stringcolumn(1), stringcolumn(2))) with boxxy lc rgb color_graph_box_new, \

