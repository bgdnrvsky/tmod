package com.tmod.core.models;

/**
 * Sent as an integer by the server
 * <p>
 * <table>
 *     <tr>
 *         <td>Type</td>
 *         <td>Value</td>
 *     </tr>
 *     <tr>
 *         <td>Release</td>
 *         <td>1</td>
 *     </tr>
 *     <tr>
 *         <td>Beta</td>
 *         <td>2</td>
 *     </tr>
 *     <tr>
 *         <td>Alpha</td>
 *         <td>3</td>
 *     </tr>
 * </table>
 */
enum ReleaseType {
    __SKIP,
    Release,
    Beta,
    Alpha,
}
