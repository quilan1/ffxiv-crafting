type FnOpt<T, U> = (_: T) => OptionType<U>;
type Fn<T, U> = (_: T) => U;

interface HasData<T> { value: NonNullable<T> };

export class OptionType<T> {
    private _isNone: boolean;
    protected value?: T;

    constructor(isNone: boolean, value?: T) {
        this._isNone = isNone;
        this.value = value;
    }

    isSome(): this is HasData<T> {
        return !this._isNone;
    }

    ////////

    map<U>(f: Fn<T, U>): OptionType<U> {
        if (!this.isSome()) return None();
        return Some(f(this.value));
    }

    mapOr<U>(d: U, f: Fn<T, U>): OptionType<U> {
        if (!this.isSome()) return Some(d);
        return Some(f(this.value));
    }

    mapOrElse<U>(d: Fn<void, U>, f: Fn<T, U>): OptionType<U> {
        if (!this.isSome()) return Some(d());
        return Some(f(this.value));
    }

    and<U>(v: OptionType<U>): OptionType<U> {
        if (!this.isSome()) return None();
        return v;
    }

    andThen<U>(f: FnOpt<T, U>): OptionType<U> {
        if (!this.isSome()) return None();
        return f(this.value);
    }

    filter(f: Fn<T, boolean>): OptionType<T> {
        if (!this.isSome()) return None();
        return !f(this.value) ? None() : this;
    }

    flatmap<U>(f: FnOpt<T, U>): OptionType<U> {
        if (!this.isSome()) return None();
        return f(this.value);
    }

    or(v: OptionType<T>): OptionType<T> {
        if (!this.isSome()) return v;
        return Some(this.value);
    }

    orElse(f: FnOpt<void, T>): OptionType<T> {
        if (!this.isSome()) return f();
        return Some(this.value);
    }

    take(): OptionType<T> {
        if (!this.isSome()) return None();
        const ret = Some(this.value);
        (this as OptionType<T>).value = undefined;
        this._isNone = true;
        return ret;
    }

    zip<U>(v: OptionType<U>): OptionType<[T, U]> {
        if (!this.isSome()) return None();
        if (!v.isSome()) return None();
        return Some([this.value as T, v.unwrap()]);
    }

    zipAll<U>(...params: OptionType<U>[]): OptionType<(U | T)[]> {
        if (!this.isSome()) return None();
        if (params.some(v => !v.isSome())) return None();
        const values = params.map(v => v.unwrapUnchecked());
        return Some([this.value, ...values]);
    }

    ////////

    flatten(): T {
        if (!this.isSome()) return None() as T;
        if (this.value instanceof OptionType) {
            return this.value as T;
        } else {
            return Some(this.value) as T;
        }
    }

    expect(msg: string): T {
        if (!this.isSome()) throw new Error(msg);
        return this.value;
    }

    unwrap(this: this extends HasData<T> ? this : never): T {
        return this.unwrapUnchecked();
    }

    unwrapUnchecked(): T {
        return this.expect("Attempting to unwrap a None value");
    }

    unwrapOr(v: T): T {
        if (!this.isSome()) return v;
        return this.value;
    }
}

export const Some = <T>(v: T) => {
    return new OptionType(false, v);
}

export const None = <T = never>() => {
    return new OptionType<T>(true);
}

export const optMin = (a: OptionType<number>, b: OptionType<number>): OptionType<number> => {
    return a.zip(b).map(([a, b]) => a < b ? a : b).or(a).or(b);
}

export const optMax = (a: OptionType<number>, b: OptionType<number>): OptionType<number> => {
    return a.zip(b).map(([a, b]) => a > b ? a : b).or(a).or(b);
}

export const optAdd = (a: OptionType<number>, b: OptionType<number>): OptionType<number> => {
    return a.zip(b).map(([a, b]) => a + b).or(a).or(b);
}

export const optSub = (a: OptionType<number>, b: OptionType<number>): OptionType<number> => {
    return a.zip(b).map(([a, b]) => a - b).or(a).or(b.map(v => -v));
}
