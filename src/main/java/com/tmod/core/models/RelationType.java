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
 *         <td>EmbeddedLibrary</td>
 *         <td>1</td>
 *     </tr>
 *     <tr>
 *         <td>OptionalDependency</td>
 *         <td>2</td>
 *     </tr>
 *     <tr>
 *         <td>RequiredDependency</td>
 *         <td>3</td>
 *     </tr>
 *     <tr>
 *         <td>Tool</td>
 *         <td>4</td>
 *     </tr>
 *     <tr>
 *         <td>Incompatible</td>
 *         <td>5</td>
 *     </tr>
 *     <tr>
 *         <td>Include</td>
 *         <td>6</td>
 *     </tr>
 * </table>
 */
public enum RelationType {
    __SKIP,
    EmbeddedLibrary,
    OptionalDependency,
    RequiredDependency,
    Tool,
    Incompatible,
    Include,
}
