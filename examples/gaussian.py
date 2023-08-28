#!/usr/bin/env python3

# hi, this is my gaussian dataclass thingy which I wrote while following the 'kalman filtering in python' guide
from dataclasses import dataclass
from typing import List
import numpy as np
from numpy.random import randn
import matplotlib.pyplot as plt
from filterpy import stats

@dataclass
class gaussian:
    mean : float
    var : float
    name : str = "gaussian"

    def __repr__(self):
        return f"{self.name}(μ = {self.mean:.3f}, var = {self.var:.3f})"

    def __add__(self, other):       
        # Addition of gaussians
        # mu = self.mean + other.mean
        # var = self.var + other.var
        if isinstance(other, gaussian):
            return gaussian(self.mean + other.mean, self.var + other.var, f"{self.name}+{other.name}")
    
    def __sub__(self, other):       
        # Subtraction of gaussians
        # mu = self.mean - other.mean
        # var = self.var + other.var
        if isinstance(other, gaussian):
            return gaussian(self.mean - other.mean, self.var + other.var, f"{self.name}-{other.name}")

    def __truediv__(self, other):
        # Useful for scaling a gaussian magnitude by a fixed value.
        # Does not affect variance.
        if isinstance(other, float):
            return gaussian(self.mean / other, self.var, f"{self.name}/{other:.2f}")
            
    def __mul__(self, other):  
        # Multiplication of gaussians - NORMALISED!
        # Use '@' (matmul) operator for the un-normalised version!
        if isinstance(other, gaussian):
            _mu = ((self.var*other.mean) + (other.var*self.mean)) / (self.var + other.var)
            _var = (self.var*other.var)/(self.var + other.var)
            return gaussian(_mu, _var, f"{self.name}*{other.name}")

    def __neg__(self):
        # Returns -1*mean equivalent of this gaussian
        return gaussian(-1*self.mean, self.var, self.name)
    
    def __mod__(self, other):
        # If we '% "text"' to a gaussian, it will replace the name!
        # This lets us write:
        # p = x + y % "prior"
        # print(p)
        # >prior(...,...)
        if isinstance(other, str):
            self.name = other
            return self

    def get_std_deviation(self,deviations):
        # Get the value which is a signed number of standard 
        # deviations above/below the mean
        return (self.mean + (self.var**.5)*deviations)
        
    @property
    def upper_end(self):
        # Returns the upper 3x standard deviation point
        return self.get_std_deviation(3)

    @property
    def lower_end(self):
        # Returns the lower 3x standard deviation point
        return self.get_std_deviation(-3)

    def __lt__(self, other):
        return self.lower_end < other.lower_end

    def __gt__(self, other):
        return self.upper_end > other.upper_end

    def __eq__(self, other):
        return ((self < other) and (self > other))


# Make a plotting function, which takes an arbitrary number of 'gaussian' items, 
# and plots them with their names on the legend :)
def plot_gaussians(gaussians : List[gaussian]):
    plt.figure()
    
    max_g = max(gaussians)
    min_g = min(gaussians)
    max_x = max_g.upper_end
    min_x = min_g.lower_end
    
    xs = np.linspace(min_x, max_x, 100)
    for g in gaussians:
        ys = [stats.gaussian(x, g.mean, g.var) for x in xs]
        plt.plot(xs, ys, label=f"{str(g)}")

    plt.legend()# Time-series line-plot of gaussian. Requires a pre-existing figure

def plot_gaussian_line(xs, g : List[gaussian], **kwargs):
    name = kwargs.get("label",g[0].name)

    y = [x.mean for x in g]

    _line, = plt.plot(xs,y,**kwargs)
    _color = facecolor=_line.get_color()

    # Fill-between is used for variance! Pretty neat!
    # This results in a nice 'gradient' of standard deviations. +/- 3,2,1..
    alpha = 0.1
    for s in range(6,0,-1):
        y_upper = [x.get_std_deviation(s/2) for x in g]
        y_lower = [x.get_std_deviation(-s/2) for x in g]   
        
        label = f"var({name})"
        
        if s != 6:
            label = "_" + label # hides the legend entries for the other two fills
            
        plt.fill_between(xs, y_lower, y_upper, alpha=alpha, label=label, facecolor=_color)
    
    