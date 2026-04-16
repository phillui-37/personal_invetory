sealed class Result<T, F> {
  const Result();

  R when<R>({
    required R Function(T value) success,
    required R Function(F failure) failure,
  }) {
    return switch (this) {
      Success<T, F>(value: final value) => success(value),
      Failure<T, F>(failure: final reason) => failure(reason),
    };
  }
}

final class Success<T, F> extends Result<T, F> {
  const Success(this.value);

  final T value;
}

final class Failure<T, F> extends Result<T, F> {
  const Failure(this.failure);

  final F failure;
}
