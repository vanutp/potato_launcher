import {useState, useEffect} from 'react';
import {CreditCard as Edit, Save, Upload, X} from 'lucide-react';

interface Modpack {
    id: string;
    name: string;
    version: string;
    loader: string;
    loaderVersion: string;
}

interface ModpackDetailsProps {
    modpack: Modpack;
    onUpdate: (id: string, updatedData: Partial<Modpack>) => void;
}

const MINECRAFT_VERSIONS = [
    '1.20.4',
    '1.20.3',
    '1.20.2',
    '1.20.1',
    '1.19.4',
    '1.19.3',
    '1.19.2',
    '1.18.2',
    '1.18.1',
];

const LOADERS = [
    {id: 'forge', name: 'Forge'},
    {id: 'neoforge', name: 'NeoForge'},
    {id: 'fabric', name: 'Fabric'},
];

const LOADER_VERSIONS = {
    forge: ['47.2.0', '47.1.3', '47.1.0', '46.0.14', '45.2.0'],
    neoforge: ['2.4.15', '2.4.10', '2.3.5', '1.7.2', '1.5.8'],
    fabric: ['0.15.6', '0.15.3', '0.14.24', '0.14.21', '0.14.19'],
};

export default function ModpackDetails({modpack, onUpdate}: ModpackDetailsProps) {
    const [isEditing, setIsEditing] = useState(false);
    const [editData, setEditData] = useState({
        name: modpack.name,
        version: modpack.version,
        loader: modpack.loader,
        loaderVersion: modpack.loaderVersion,
    });
    const [dragActive, setDragActive] = useState(false);
    const [uploadedFiles, setUploadedFiles] = useState<FileList | null>(null);

    useEffect(() => {
        setIsEditing(false);
        setUploadedFiles(null);
        setEditData({
            name: modpack.name,
            version: modpack.version,
            loader: modpack.loader,
            loaderVersion: modpack.loaderVersion,
        });
    }, [modpack.id]);

    const handleEdit = () => {
        setIsEditing(true);
        setEditData({
            name: modpack.name,
            version: modpack.version,
            loader: modpack.loader,
            loaderVersion: modpack.loaderVersion,
        });
    };

    const handleCancel = () => {
        setIsEditing(false);
        setUploadedFiles(null);
        setEditData({
            name: modpack.name,
            version: modpack.version,
            loader: modpack.loader,
            loaderVersion: modpack.loaderVersion,
        });
    };

    const handleUpdate = () => {
        onUpdate(modpack.id, editData);
        setIsEditing(false);
        setUploadedFiles(null);
    };

    const handleInputChange = (field: keyof typeof editData, value: string) => {
        setEditData(prev => ({...prev, [field]: value}));
    };

    const handleDrag = (e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.type === 'dragenter' || e.type === 'dragover') {
            setDragActive(true);
        } else if (e.type === 'dragleave') {
            setDragActive(false);
        }
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setDragActive(false);

        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
            setUploadedFiles(e.dataTransfer.files);
        }
    };

    const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files[0]) {
            setUploadedFiles(e.target.files);
        }
    };

    return (
        <div className="max-w-2xl mx-auto">
            <div className="bg-gray-800 rounded-lg border border-gray-700 p-8">
                <div className="flex items-center justify-between mb-6">
                    <h2 className="text-2xl font-bold text-white">
                        {isEditing ? 'Edit Modpack' : modpack.name}
                    </h2>
                    {!isEditing && (
                        <button
                            onClick={handleEdit}
                            className="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-md transition-colors duration-200"
                        >
                            <Edit size={16}/>
                            Update modpack
                        </button>
                    )}
                </div>

                <div className="space-y-6">
                    {isEditing ? (
                        <>
                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Modpack Name *
                                </label>
                                <input
                                    type="text"
                                    value={editData.name}
                                    onChange={(e) => handleInputChange('name', e.target.value)}
                                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Minecraft Version *
                                </label>
                                <select
                                    value={editData.version}
                                    onChange={(e) => handleInputChange('version', e.target.value)}
                                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                                >
                                    {MINECRAFT_VERSIONS.map((version) => (
                                        <option key={version} value={version}>
                                            {version}
                                        </option>
                                    ))}
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Mod Loader *
                                </label>
                                <select
                                    value={editData.loader}
                                    onChange={(e) => handleInputChange('loader', e.target.value)}
                                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                                >
                                    {LOADERS.map((loader) => (
                                        <option key={loader.id} value={loader.id}>
                                            {loader.name}
                                        </option>
                                    ))}
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Loader Version *
                                </label>
                                <select
                                    value={editData.loaderVersion}
                                    onChange={(e) => handleInputChange('loaderVersion', e.target.value)}
                                    className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200"
                                >
                                    {LOADER_VERSIONS[editData.loader as keyof typeof LOADER_VERSIONS]?.map((version) => (
                                        <option key={version} value={version}>
                                            {version}
                                        </option>
                                    ))}
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    Upload Files (Optional)
                                </label>
                                <div
                                    className={`relative border-2 border-dashed rounded-lg p-8 text-center transition-all duration-200 ${
                                        dragActive
                                            ? 'border-green-500 bg-green-500/10'
                                            : 'border-gray-600 hover:border-gray-500'
                                    }`}
                                    onDragEnter={handleDrag}
                                    onDragLeave={handleDrag}
                                    onDragOver={handleDrag}
                                    onDrop={handleDrop}
                                >
                                    <input
                                        type="file"
                                        multiple
                                        onChange={handleFileInput}
                                        className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                                        webkitdirectory=""
                                    />
                                    <Upload className="mx-auto h-12 w-12 text-gray-400 mb-4"/>
                                    <p className="text-gray-300 mb-2">
                                        Drag and drop modpack folder here, or click to browse
                                    </p>
                                    <p className="text-sm text-gray-500">
                                        Upload your modpack files and folders
                                    </p>
                                    {uploadedFiles && (
                                        <div className="mt-4 p-3 bg-gray-700 rounded-md">
                                            <p className="text-green-400 text-sm">
                                                {uploadedFiles.length} file(s) selected
                                            </p>
                                        </div>
                                    )}
                                </div>
                            </div>

                            <div className="flex gap-3 pt-4">
                                <button
                                    onClick={handleUpdate}
                                    className="flex items-center gap-2 px-6 py-3 bg-green-500 hover:bg-green-600 text-white font-medium rounded-md transition-colors duration-200"
                                >
                                    <Save size={16}/>
                                    Update
                                </button>
                                <button
                                    onClick={handleCancel}
                                    className="flex items-center gap-2 px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-medium rounded-md transition-colors duration-200"
                                >
                                    <X size={16}/>
                                    Cancel
                                </button>
                            </div>
                        </>
                    ) : (
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-400 mb-1">
                                    Minecraft Version
                                </label>
                                <p className="text-white bg-gray-700 px-4 py-2 rounded-md">
                                    {modpack.version}
                                </p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-400 mb-1">
                                    Mod Loader
                                </label>
                                <p className="text-white bg-gray-700 px-4 py-2 rounded-md capitalize">
                                    {modpack.loader}
                                </p>
                            </div>
                            <div className="col-span-2">
                                <label className="block text-sm font-medium text-gray-400 mb-1">
                                    Loader Version
                                </label>
                                <p className="text-white bg-gray-700 px-4 py-2 rounded-md">
                                    {modpack.loaderVersion}
                                </p>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}