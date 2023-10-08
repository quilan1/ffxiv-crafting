type FnOpt<T, U> = (_: T) => OptionType<U>;
type Fn<T, U> = (_: T) => U;

interface HasData<T> { value: NonNullable<T> };

export class OptionType<T> {
    private _is_none: boolean;
    protected value?: T;

    constructor(is_none: boolean, value?: T) {
        this._is_none = is_none;
        this.value = value;
    }

    is_some(): this is HasData<T> {
        return !this._is_none;
    }

    ////////

    map<U>(f: Fn<T, U>): OptionType<U> {
        if (!this.is_some()) return None();
        return Some(f(this.value));
    }

    map_or<U>(d: U, f: Fn<T, U>): OptionType<U> {
        if (!this.is_some()) return Some(d);
        return Some(f(this.value));
    }

    map_or_else<U>(d: Fn<void, U>, f: Fn<T, U>): OptionType<U> {
        if (!this.is_some()) return Some(d());
        return Some(f(this.value));
    }

    and<U>(v: OptionType<U>): OptionType<U> {
        if (!this.is_some()) return None();
        return v;
    }

    and_then<U>(f: FnOpt<T, U>): OptionType<U> {
        if (!this.is_some()) return None();
        return f(this.value);
    }

    filter(f: Fn<T, boolean>): OptionType<T> {
        if (!this.is_some()) return None();
        return !f(this.value) ? None() : this;
    }

    flatmap<U>(f: FnOpt<T, U>): OptionType<U> {
        if (!this.is_some()) return None();
        return f(this.value);
    }

    or(v: OptionType<T>): OptionType<T> {
        if (!this.is_some()) return v;
        return Some(this.value);
    }

    or_else(f: FnOpt<void, T>): OptionType<T> {
        if (!this.is_some()) return f();
        return Some(this.value);
    }

    take(): OptionType<T> {
        if (!this.is_some()) return None();
        const ret = Some(this.value);
        (this as OptionType<T>).value = undefined;
        this._is_none = true;
        return ret;
    }

    zip<U>(v: OptionType<U>): OptionType<[T, U]> {
        if (!this.is_some()) return None();
        if (!v.is_some()) return None();
        return Some([this.value as T, v.unwrap()]);
    }

    zip_all<U>(...params: OptionType<U>[]): OptionType<(U|T)[]> {
        if (!this.is_some()) return None();
        if (params.some(v => !v.is_some())) return None();
        const values = params.map(v => v.unwrap_unchecked());
        return Some([this.value, ...values]);
    }

    ////////

    flatten(): T {
        if (!this.is_some()) return None() as T;
        if (this.value instanceof OptionType) {
            return this.value as T;
        } else {
            return Some(this.value) as T;
        }
    }

    expect(msg: string): T {
        if (!this.is_some()) throw new Error(msg);
        return this.value;
    }

    unwrap(this: this extends HasData<T> ? this : never): T {
        return this.unwrap_unchecked();
    }

    unwrap_unchecked(): T {
        return this.expect("Attempting to unwrap a None value");
    }

    unwrap_or(v: T): T {
        if (!this.is_some()) return v;
        return this.value;
    }
}

export const Some = <T>(v: T) => {
    return new OptionType(false, v);
}

export const None = <T=never>() => {
    return new OptionType<T>(true);
}
