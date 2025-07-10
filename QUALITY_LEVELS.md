# Quality levels

The quality levels for the components in iceoryx2 are derived from the
[ROS quality levels](https://github.com/ros-infrastructure/rep/blob/master/rep-2004.rst).
Despite developing some targets according to automotive standards like ISO26262,
the code base standalone does NOT legitimize the usage in a safety-critical
system. All requirements of a lower quality level are included in higher quality
levels e.g. quality level 4 is included in quality level 3.

## Quality level 5

This quality level is the default quality level. It is meant for examples and
helper tools.

* Derived from [ROS quality level 5](https://www.ros.org/reps/rep-2004.html#quality-level-5)
    * Reviewed by at least one approver
    * No compiler warnings
    * License and copyright statements available
    * No version policy required
    * No unit tests required

## Quality level 4

* Derived from [ROS quality level 4](https://www.ros.org/reps/rep-2004.html#quality-level-4)
    * Basic unit tests are required
    * Builds and runs on Windows, macOS, Linux

## Quality level 3

* Derived from [ROS quality level 3](https://www.ros.org/reps/rep-2004.html#quality-level-3)
    * Documentation required
    * Test specification required
    * Version policy required

## Quality level 2

This quality level is meant for all targets that need tier 1 support in ROS 2.

* Derived from [ROS quality level 2](https://www.ros.org/reps/rep-2004.html#quality-level-2)
    * Must have a [quality declaration document](https://www.ros.org/reps/rep-2004.html#quality-declaration-template)

## Quality level 1

* Derived from [ROS quality level 1](https://www.ros.org/reps/rep-2004.html#quality-level-1)
    * Version policy for stable API and ABI required
    * [ASPICE](https://beza1e1.tuxen.de/aspice.html) SWE.6 tests available
    * Performance tests and regression policy required
    * Static code analysis warnings addressed
    * Enforcing the code style is required
    * Unit tests have full statement and branch coverage

## Quality level 1+

This quality level goes beyond the ROS quality levels and contains extensions.

* Code coverage according to
[MC/DC](https://en.wikipedia.org/wiki/Modified_condition/decision_coverage) available
