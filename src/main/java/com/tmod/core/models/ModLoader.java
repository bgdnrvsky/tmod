package com.tmod.core.models;

// NOTE: Its own file, because will be used by the pool

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * Sent as an integer by the server
 * <p>
 * <table>
 *     <tr>
 *         <td>Loader</td>
 *         <td>Value</td>
 *     </tr>
 *     <tr>
 *         <td>Forge</td>
 *         <td>1</td>
 *     </tr>
 *     <tr>
 *         <td>Fabric</td>
 *         <td>4</td>
 *     </tr>
 *     <tr>
 *         <td>Quilt</td>
 *         <td>5</td>
 *     </tr>
 *     <tr>
 *         <td>NeoForge</td>
 *         <td>6</td>
 *     </tr>
 * </table>
 */
public enum ModLoader {
    Forge(1),
    Fabric(4),
    Quilt(5),
    NeoForge(6);

    private final int id;

    ModLoader(int id) {
        this.id = id;
    }

    @JsonValue
    public int getId() {
        return id;
    }
}
