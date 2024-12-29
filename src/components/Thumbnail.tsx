export function Thumbnail({src: url, className}: {src: string, className?: string}) {
    return <div className={"bg-center bg-cover bg-no-repeat rounded " + className} style={{backgroundImage: `url(${url})`}}>

    </div>
}
