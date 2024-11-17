package com.tmod.core.models;

// NOTE: Its own file, because will be used by the pool

/**
 * Sent as an integer by the server
 * <p>
 * <table>
 *     <tr>
 *         <td>Loader</td>
 *         <td>Value</td>
 *     </tr>
 *     <tr>
 *         <td>Any</td>
 *         <td>0</td>
 *     </tr>
 *     <tr>
 *         <td>Forge</td>
 *         <td>1</td>
 *     </tr>
 *     <tr>
 *         <td>Cauldron</td>
 *         <td>2</td>
 *     </tr>
 *     <tr>
 *         <td>LiteLoader</td>
 *         <td>3</td>
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
    Any,
    Forge,
    Cauldron,
    LiteLoader,
    Fabric,
    Quilt,
    NeoForge
}
