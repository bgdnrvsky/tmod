package com.tmod.core.models;

/**
 * This class is just a model to easily unwrap inner model from CurseForge API reponses,
 * because JSON object returned by the API always contain a "data" key.
 *
 * @param <T> Inner model contained by the response
 */
public class JsonResponse<T> {

    private T data;

    public T getData() {
        return data;
    }
}
