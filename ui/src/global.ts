interface Array<T> {
    equals(other: T[]): boolean;
}

Array.prototype.equals = function (other) {
    if (this.length !== other.length) return false;
    for (let i = 0; i < this.length; i++)
        if (this[i] !== other[i])
            return false;
    return true;
}