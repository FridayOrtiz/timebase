---
title: Title Goes Here
author:
  - "Rafael Ortiz &lt;rortiz12@jhu.edu&gt;"
  - "Ramon Benitez-Pagan &lt;ramon.benitez@jhu.edu&gt;"
  - "Stephen Scally &lt;sscally@jhu.edu&gt;"
  - "Cyrus Bulsara &lt;cbulsar1@jhu.edu&gt;"
header-includes:
  - \usepackage[ruled,vlined,linesnumbered]{algorithm2e}
  - \usepackage{tikz}
  - \usepackage{gensymb}
  - \usepackage{fancyhdr}
  - \usepackage{amsmath}
  - \usepackage{stmaryrd}
  - \usepackage{lastpage}
  - \usepackage{multicol}
  - \newcommand{\hideFromPandoc}[1]{#1}
  - \hideFromPandoc{
      \let\Begin\begin
      \let\End\end
    }
  - \pagestyle{fancy}
  - \fancyfoot[LO,LE]{Ortiz R., Benitez R., Scally S., Bulsara C.}
  - \fancyfoot[C]{}
  - \fancyfoot[RE,RO]{\thepage\ of \pageref{LastPage}}
  - \renewcommand*{\thefootnote}{\normalfont\fnsymbol{footnote}} 
bibliography: paper.bib
type: pdf\_document
geometry: margin=3cm
toc: false
---


\Begin{multicols}{2}

# Abstract

Some abstract stuff here. This is an example of how to insert a reference to the example paper [@ExamplePaper].

# Introduction

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed varius iaculis sem at malesuada. Maecenas consequat sem in tellus eleifend tempor. Nam vel posuere ipsum. Etiam scelerisque aliquam tellus, molestie aliquet orci porta id. Aliquam a ultrices ligula. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Fusce lobortis urna in pellentesque suscipit. Fusce sed pharetra tortor. Curabitur quis leo quis elit pulvinar lacinia. Suspendisse nibh elit, molestie fringilla suscipit et, fringilla quis sapien. Maecenas nec suscipit leo.

# Selected Covert Channels

Nullam laoreet et nisl sed ornare. Vivamus eu neque vitae lorem faucibus posuere ut quis est. Aenean congue urna at mauris auctor, ac mattis erat faucibus. Fusce convallis elit dui, id rutrum quam ultricies vel. Aliquam cursus odio lacus. Integer cursus mauris ipsum, vitae vehicula turpis vehicula a. Nunc rutrum a felis a ultrices. Vestibulum et porttitor tortor, ut ultricies libero. Integer quis massa eu libero cursus egestas. Vestibulum urna mi, congue id congue non, euismod vitae est. Nam sit amet rhoncus augue. Donec molestie metus at sem faucibus, et pellentesque ex rhoncus. Donec aliquet quis urna sed mattis. Maecenas sit amet augue molestie, pharetra est eu, facilisis dui. Quisque lobortis suscipit libero vel hendrerit.

# Detection Methodology

Pellentesque non condimentum nisl. Integer pretium dolor quis consectetur venenatis. Donec ut nibh volutpat elit vestibulum scelerisque. In cursus orci nec bibendum cursus. Donec finibus odio nisi, vel posuere mauris luctus non. Nam vel viverra dui, in fringilla lacus. Sed sapien massa, ultrices quis fringilla vel, hendrerit a diam. Donec ornare urna ut lacus convallis auctor. Etiam auctor ligula ultrices elementum hendrerit. Curabitur nec tempor elit. Vestibulum in enim eu eros condimentum sodales faucibus venenatis velit. Nunc sed mollis erat. Nulla a ante ligula. Cras nulla ligula, venenatis sed placerat id, blandit vel mauris. Vestibulum eu justo malesuada, ornare dolor eu, sollicitudin nunc.

# Implementation

Donec odio tortor, cursus et vestibulum ac, rutrum quis felis. Suspendisse elit sem, consectetur in nisi ut, blandit faucibus dui. Nullam aliquet, neque feugiat dictum congue, justo magna pulvinar ante, ut maximus lectus ligula eu tortor. Integer consequat molestie rhoncus. Praesent accumsan odio aliquam malesuada tincidunt. Nunc sed sollicitudin sapien. Proin at nisl lacus. Etiam blandit dapibus convallis. Sed in luctus tortor, sed tempor leo. Interdum et malesuada fames ac ante ipsum primis in faucibus. Nulla ultrices est turpis, eu hendrerit orci ullamcorper venenatis. Duis sed ligula sed nulla faucibus accumsan. Nam malesuada tortor nec condimentum ultricies. Cras vehicula sem sed libero tempor fermentum. Donec feugiat erat sed dolor vehicula ornare. Maecenas ut justo ut ante semper porttitor.

# Evaluation

Donec cursus dui a nisl posuere sagittis. Aliquam et pellentesque dolor, sit amet tincidunt nisl. Sed porta condimentum nibh id posuere. Aliquam erat volutpat. Maecenas suscipit, ex vitae euismod suscipit, nisi orci auctor quam, at interdum nulla libero tincidunt quam. Nam tempus blandit aliquet. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur vitae lacus et diam faucibus scelerisque. Vivamus ut tristique magna, vel venenatis augue. Nullam odio turpis, vestibulum eu urna in, aliquam scelerisque nibh. Sed et turpis nec quam convallis vulputate et ac sem.

# Discussion and Future Work

Donec cursus dui a nisl posuere sagittis. Aliquam et pellentesque dolor, sit amet tincidunt nisl. Sed porta condimentum nibh id posuere. Aliquam erat volutpat. Maecenas suscipit, ex vitae euismod suscipit, nisi orci auctor quam, at interdum nulla libero tincidunt quam. Nam tempus blandit aliquet. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur vitae lacus et diam faucibus scelerisque. Vivamus ut tristique magna, vel venenatis augue. Nullam odio turpis, vestibulum eu urna in, aliquam scelerisque nibh. Sed et turpis nec quam convallis vulputate et ac sem.



\pagebreak
# References

\bibliography{paper}


\End{multicols}
