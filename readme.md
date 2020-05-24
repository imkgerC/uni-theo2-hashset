# Hashset Testing
This repository was created to collect my solution to an exercise given at DHBW Karlsruhe. The purpose was to find the average number of collisions for different configurations of hashsets. A big bunch of variants was implemented but only a subset of those is shown in those graphs. For more please run the code yourself.
## Memory fairness
As the original task asked to fix the number of buckets and use this same number for every type of hashset it heavily favors more memory-heavy variants of hashsets. Because of this problem the option `RESIZE_TO_MAKE_FAIR` was introduced. It tries to scale every type of table in the most fair way possible. Through this method the following graphs were obtained.

If you would like to know how the amount of buckets per hashtable type was calculated you can read the pdf-File at latex/main.pdf (It is writting in German, if you would like an explanation in English, feel free to contact me).

![graph success time](https://github.com/imkgerC/uni-theo2-hashset/blob/master/graphs/successful_time.png)

![graph failure time](https://github.com/imkgerC/uni-theo2-hashset/blob/master/graphs/failure_time.png)

![graph failure collisions](https://github.com/imkgerC/uni-theo2-hashset/blob/master/graphs/failure_collisions.png)

![graph success collisions](https://github.com/imkgerC/uni-theo2-hashset/blob/master/graphs/successful_collisions.png)
## Using this code anywhere serious
Please do not. This is just a quickly written study of different hashing and collision resolution methods.